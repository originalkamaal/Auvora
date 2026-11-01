//! Tenant-isolation property tests: the P0 exit criterion in
//! `docs/specs/tenancy-rls.md` §8.
//!
//! One PostgreSQL 16 container is shared by all tests in this binary. Each
//! test builds its own tenants and projects with fresh IDs, so tests can run
//! in parallel without seeing each other's data as "foreign" by accident.
//!
//! P8 (platform scope audit) and the audit chain are covered at the end.

// Test harness: failing fast with a message is the desired behaviour here
// (ADR-0047 allows unwrap/expect/panic in tests; helpers aren't `#[test]` fns).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::{BTreeSet, HashSet};
use std::sync::{Mutex, OnceLock};

use auvora_core::id::{Id, Kind};
use auvora_core::time::{Clock, SystemClock};
use auvora_store::platform::{self, ActorRef, PlatformReason, PlatformScope, PlatformStore};
use auvora_store::repo::audit::{self, ChainError, NewAuditEvent};
use auvora_store::repo::identifiers::{self, NewIdentifier};
use auvora_store::repo::principals::{self, PrincipalKind};
use auvora_store::repo::{environment_groups, jobs, outbox, projects, tenants, users};
use auvora_store::{BOOTSTRAP_SQL, MIGRATOR, Store, StoreError, TenantScope, TenantTx};
use proptest::prelude::*;
use serde_json::json;
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{AssertSqlSafe, Row};
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::runners::SyncRunner;
use testcontainers_modules::testcontainers::{Container, ImageExt};
use uuid::Uuid;

const PASSWORD: &str = "test-only-password";
/// Tables outside tenant RLS (cell-local platform allowlist, tenancy-rls.md §5).
const PLATFORM_TABLES: &[&str] = &["_sqlx_migrations", "platform_audit_events"];
/// `SECURITY DEFINER` functions allowed in schema `auvora` (tenancy-rls.md §5).
const DEFINER_FUNCTIONS: &[&str] = &["create_partitions"];

// ---------------------------------------------------------------- fixture

struct Fixture {
    _container: Container<Postgres>,
    port: u16,
}

impl Fixture {
    fn url(&self, role: &str) -> String {
        let password = if role == "postgres" {
            "postgres"
        } else {
            PASSWORD
        };
        format!(
            "postgres://{role}:{password}@127.0.0.1:{}/postgres",
            self.port
        )
    }
}

fn fixture() -> &'static Fixture {
    static FIXTURE: OnceLock<Fixture> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        // The sync runner owns its own runtime, so start it off any test runtime.
        std::thread::spawn(|| {
            let container = Postgres::default()
                .with_tag("16-alpine")
                .start()
                .expect("start postgres");
            let port = container.get_host_port_ipv4(5432).expect("postgres port");
            let fixture = Fixture {
                _container: container,
                port,
            };
            runtime().block_on(prepare_database(&fixture));
            fixture
        })
        .join()
        .expect("fixture thread")
    })
}

async fn prepare_database(fx: &Fixture) {
    let admin = PgPool::connect(&fx.url("postgres"))
        .await
        .expect("connect as superuser");
    sqlx::raw_sql(BOOTSTRAP_SQL)
        .execute(&admin)
        .await
        .expect("bootstrap");
    for role in ["auvora_owner", "auvora_app", "auvora_platform"] {
        let sql = format!("ALTER ROLE {role} LOGIN PASSWORD '{PASSWORD}'");
        sqlx::raw_sql(AssertSqlSafe(sql))
            .execute(&admin)
            .await
            .expect("set password");
    }
    let owner = PgPool::connect(&fx.url("auvora_owner"))
        .await
        .expect("connect as owner");
    MIGRATOR.run(&owner).await.expect("migrations");
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime")
}

async fn store() -> Store {
    Store::connect(&fixture().url("auvora_app"), 4)
        .await
        .expect("store")
}

/// Ground truth: the RLS-bypassing platform role.
async fn platform_pool() -> PgPool {
    PgPool::connect(&fixture().url("auvora_platform"))
        .await
        .expect("platform pool")
}

/// A raw `auvora_app` pool for statements the `Store` deliberately can't make.
async fn raw_app_pool(max: u32) -> PgPool {
    PgPoolOptions::new()
        .max_connections(max)
        .connect(&fixture().url("auvora_app"))
        .await
        .expect("raw app pool")
}

fn new_id(kind: Kind) -> Id {
    Id::generate(kind, &SystemClock).expect("id")
}

// ---------------------------------------------------------------- world

/// Two tenants: A owns projects A1 and A2; B owns B1 (tenancy-rls.md §8).
#[derive(Debug, Clone)]
struct World {
    a: Id,
    b: Id,
    a1: Id,
    a2: Id,
    b1: Id,
    /// Principals per project: (project, principal UUID).
    principals: Vec<(Id, Uuid)>,
    /// Identifiers per project: (project, identifier UUID, principal UUID, blind index).
    identifiers: Vec<(Id, Uuid, Uuid, Vec<u8>)>,
}

impl World {
    fn scope(&self, project: Id) -> TenantScope {
        let tenant = if project == self.b1 { self.b } else { self.a };
        TenantScope::project(tenant, project).expect("scope")
    }

    fn principals_in(&self, project: Id) -> Vec<Uuid> {
        self.principals
            .iter()
            .filter(|(p, _)| *p == project)
            .map(|(_, id)| *id)
            .collect()
    }

    fn identifiers_in(&self, project: Id) -> Vec<(Uuid, Uuid, Vec<u8>)> {
        self.identifiers
            .iter()
            .filter(|(p, ..)| *p == project)
            .map(|(_, id, principal, bidx)| (*id, *principal, bidx.clone()))
            .collect()
    }
}

/// A project's random contents: principal kinds, and identifier values drawn
/// from a tiny shared alphabet so values collide across projects on purpose.
#[derive(Debug, Clone)]
struct ProjectSpec {
    kinds: Vec<PrincipalKind>,
    values: Vec<u8>,
}

fn project_spec() -> impl Strategy<Value = ProjectSpec> {
    (
        prop::collection::vec(prop::sample::select(PrincipalKind::ALL.to_vec()), 1..5),
        prop::collection::vec(0u8..6, 0..6),
    )
        .prop_map(|(kinds, values)| ProjectSpec { kinds, values })
}

async fn build_world(store: &Store, specs: [&ProjectSpec; 3]) -> World {
    let (a, b) = (new_id(Kind::Tenant), new_id(Kind::Tenant));
    let (a1, a2, b1) = (
        new_id(Kind::Project),
        new_id(Kind::Project),
        new_id(Kind::Project),
    );

    for (tenant, projects) in [(a, vec![a1, a2]), (b, vec![b1])] {
        let mut tx = store
            .begin(&TenantScope::tenant(tenant).unwrap())
            .await
            .unwrap();
        tenants::create(&mut tx, &tenant.to_string(), "tenant")
            .await
            .unwrap();
        environment_groups::create(&mut tx, new_id(Kind::EnvironmentLink), "product")
            .await
            .unwrap();
        for project in projects {
            // Same slug in every tenant on purpose (uniqueness is per tenant).
            let slug = if project == a2 { "second" } else { "main" };
            projects::create(
                &mut tx,
                project,
                slug,
                "project",
                &format!("https://{project}.test"),
            )
            .await
            .unwrap();
        }
        tx.commit().await.unwrap();
    }

    let mut world = World {
        a,
        b,
        a1,
        a2,
        b1,
        principals: vec![],
        identifiers: vec![],
    };
    for (project, spec) in [a1, a2, b1].into_iter().zip(specs) {
        let mut tx = store.begin(&world.scope(project)).await.unwrap();
        let mut ids = vec![];
        for kind in &spec.kinds {
            let p = principals::create(&mut tx, new_id(kind.id_kind()), *kind)
                .await
                .unwrap();
            ids.push(p.id.uuid());
            world.principals.push((project, p.id.uuid()));
            if *kind == PrincipalKind::User {
                users::create(&mut tx, p.id.uuid()).await.unwrap();
            }
            // ADR-0039/0008: audit and outbox rows commit with the change.
            let event = NewAuditEvent {
                category: "admin",
                action: "principal.created",
                outcome: "success",
                actor: json!({"type": "system", "id": "test"}),
                subject: json!({"type": "principal", "id": p.id.to_string()}),
                metadata: json!({}),
            };
            audit::append(&mut tx, &event, &SystemClock).await.unwrap();
            outbox::enqueue(
                &mut tx,
                "auvora.principal.created.v1",
                &p.id.to_string(),
                &json!({}),
                &SystemClock,
            )
            .await
            .unwrap();
        }
        // Jobs scheduled an hour ahead, so only P8's own job is ever due.
        let run_at = SystemClock.unix_micros() + 3_600_000_000;
        jobs::enqueue(&mut tx, new_id(Kind::Job), "test.noop", &json!({}), run_at)
            .await
            .unwrap();
        let mut seen = HashSet::new();
        for (i, value) in spec.values.iter().enumerate() {
            if !seen.insert(*value) {
                continue; // unique within a project; duplicates are tested in P6
            }
            let principal = ids[i % ids.len()];
            let bidx = vec![*value; 32];
            let new = NewIdentifier {
                id: new_id(Kind::Identifier),
                principal_id: principal,
                kind: "email",
                value_normalized_bidx: &bidx,
                value_display: Some("someone@example.test"),
            };
            let created = identifiers::create(&mut tx, &new).await.unwrap();
            world
                .identifiers
                .push((project, created.id.uuid(), principal, bidx));
        }
        tx.commit().await.unwrap();
    }
    world
}

/// Order-independent fingerprint of everything a project owns, read as platform.
async fn project_checksum(platform: &PgPool, project: Id) -> String {
    sqlx::query_scalar(
        "SELECT coalesce(string_agg(h, ',' ORDER BY h), '') FROM ( \
           SELECT md5(p::text) AS h FROM auvora.principals p WHERE project_id = $1 \
           UNION ALL SELECT md5(i::text) FROM auvora.identifiers i WHERE project_id = $1 \
           UNION ALL SELECT md5(u::text) FROM auvora.users u WHERE project_id = $1 \
           UNION ALL SELECT md5(a::text) FROM auvora.audit_events a WHERE project_id = $1 \
           UNION ALL SELECT md5(o::text) FROM auvora.outbox_events o WHERE project_id = $1 \
           UNION ALL SELECT md5(j::text) FROM auvora.jobs j WHERE project_id = $1) x",
    )
    .bind(project.uuid())
    .fetch_one(platform)
    .await
    .unwrap()
}

// ---------------------------------------------------------------- catalog helpers

/// Tenant tables discovered from the catalog (never hand-listed), with their level.
async fn tenant_tables(platform: &PgPool) -> Vec<(String, &'static str)> {
    let rows = sqlx::query(
        "SELECT c.relname, EXISTS (SELECT 1 FROM pg_attribute a WHERE a.attrelid = c.oid \
                AND a.attname = 'project_id' AND NOT a.attisdropped) AS project_level \
         FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace \
         WHERE n.nspname = 'auvora' AND c.relkind IN ('r', 'p') AND NOT c.relispartition \
         ORDER BY c.relname",
    )
    .fetch_all(platform)
    .await
    .unwrap();
    rows.into_iter()
        .map(|r| {
            (
                r.get::<String, _>(0),
                if r.get::<bool, _>(1) {
                    "project"
                } else {
                    "tenant"
                },
            )
        })
        .filter(|(name, _)| !PLATFORM_TABLES.contains(&name.as_str()))
        .collect()
}

// ---------------------------------------------------------------- coverage gate

static EXERCISED: Mutex<BTreeSet<&'static str>> = Mutex::new(BTreeSet::new());

fn hit(method: &'static str) {
    EXERCISED.lock().unwrap().insert(method);
}

/// Every `pub async fn` in `src/repo/*.rs`, found by scanning the sources.
fn repository_methods() -> BTreeSet<String> {
    let sources = [
        ("audit", include_str!("../src/repo/audit.rs")),
        (
            "environment_groups",
            include_str!("../src/repo/environment_groups.rs"),
        ),
        ("identifiers", include_str!("../src/repo/identifiers.rs")),
        ("jobs", include_str!("../src/repo/jobs.rs")),
        ("outbox", include_str!("../src/repo/outbox.rs")),
        ("principals", include_str!("../src/repo/principals.rs")),
        ("projects", include_str!("../src/repo/projects.rs")),
        ("tenants", include_str!("../src/repo/tenants.rs")),
        ("users", include_str!("../src/repo/users.rs")),
    ];
    let mut methods = BTreeSet::new();
    for (module, source) in sources {
        for line in source.lines() {
            if let Some(rest) = line.trim_start().strip_prefix("pub async fn ") {
                let name = rest.split(['(', '<']).next().unwrap();
                methods.insert(format!("{module}::{name}"));
            }
        }
    }
    methods
}

#[test]
fn repository_sources_list_matches_repo_modules() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/repo");
    let mut on_disk: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|f| f != "mod.rs")
        .collect();
    on_disk.sort();
    // If this fails, add the new module to `repository_methods` above.
    assert_eq!(
        on_disk,
        [
            "audit.rs",
            "environment_groups.rs",
            "identifiers.rs",
            "jobs.rs",
            "outbox.rs",
            "principals.rs",
            "projects.rs",
            "tenants.rs",
            "users.rs"
        ]
    );
}

// ---------------------------------------------------------------- properties

/// P1 + P2 + coverage gate: reads under scope A1 only ever see A1, writes
/// aimed at B1 rows change nothing, and every repository method is exercised.
#[test]
fn p1_p2_read_and_write_isolation_cover_every_repository_method() {
    let rt = runtime();
    let (store, platform) = rt.block_on(async { (store().await, platform_pool().await) });
    // 256 cases by default (tenancy-rls.md §8); nightly sets PROPTEST_CASES=10000.
    let config = ProptestConfig::default();

    proptest!(config, |(s1 in project_spec(), s2 in project_spec(), s3 in project_spec())| {
        rt.block_on(check_isolation_case(&store, &platform, [&s1, &s2, &s3]));
    });

    let exercised: BTreeSet<String> = EXERCISED
        .lock()
        .unwrap()
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    let missing: Vec<_> = repository_methods()
        .difference(&exercised)
        .cloned()
        .collect();
    assert!(
        missing.is_empty(),
        "repository methods not exercised by P1/P2: {missing:?}"
    );
}

/// One P1/P2 case: build a random world, then check reads and writes under scope A1.
async fn check_isolation_case(store: &Store, platform: &PgPool, specs: [&ProjectSpec; 3]) {
    let w = build_world(store, specs).await;
    hit("tenants::create");
    hit("projects::create");
    hit("principals::create");
    hit("identifiers::create");
    let before = project_checksum(platform, w.b1).await;
    let mut tx = store.begin(&w.scope(w.a1)).await.unwrap();

    check_reads(&mut tx, &w, platform).await;
    check_writes(&mut tx, &w).await;
    tx.commit().await.unwrap();

    assert_eq!(project_checksum(platform, w.b1).await, before, "B1 changed");
}

/// P1: under scope A1, every repository read sees only A1 (or tenant A) rows.
async fn check_reads(tx: &mut TenantTx, w: &World, platform: &PgPool) {
    assert_eq!(tenants::get(tx).await.unwrap().id, w.a);
    hit("tenants::get");
    let visible: Vec<Id> = projects::list(tx)
        .await
        .unwrap()
        .into_iter()
        .map(|p| p.id)
        .collect();
    assert_eq!(
        visible.iter().copied().collect::<HashSet<_>>(),
        HashSet::from([w.a1, w.a2])
    );
    hit("projects::list");
    assert!(matches!(
        projects::get(tx, w.b1).await,
        Err(StoreError::NotFound)
    ));
    assert_eq!(projects::get(tx, w.a2).await.unwrap().id, w.a2);
    hit("projects::get");

    let listed = principals::list(tx).await.unwrap();
    assert!(listed.iter().all(|p| p.project_id == w.a1));
    assert_eq!(listed.len(), w.principals_in(w.a1).len());
    hit("principals::list");
    for id in w
        .principals_in(w.b1)
        .into_iter()
        .chain(w.principals_in(w.a2))
    {
        assert!(matches!(
            principals::get(tx, id).await,
            Err(StoreError::NotFound)
        ));
        assert_eq!(
            identifiers::list_for_principal(tx, id).await.unwrap(),
            Vec::new()
        );
    }
    for id in w.principals_in(w.a1) {
        assert_eq!(principals::get(tx, id).await.unwrap().project_id, w.a1);
    }
    hit("principals::get");
    hit("identifiers::list_for_principal");
    let a1_values: HashSet<Vec<u8>> = w
        .identifiers_in(w.a1)
        .into_iter()
        .map(|(.., b)| b)
        .collect();
    for (_, _, bidx) in w.identifiers_in(w.b1) {
        match identifiers::find_by_value(tx, "email", &bidx).await {
            Ok(found) => {
                assert_eq!(found.project_id, w.a1);
                assert!(a1_values.contains(&bidx));
            }
            Err(StoreError::NotFound) => assert!(!a1_values.contains(&bidx)),
            Err(e) => panic!("unexpected error: {e}"),
        }
    }
    hit("identifiers::find_by_value");

    let groups = environment_groups::list(tx).await.unwrap();
    assert!(groups.len() == 1 && groups.iter().all(|g| g.tenant_id == w.a.uuid()));
    hit("environment_groups::create");
    hit("environment_groups::list");
    for id in w.principals_in(w.b1) {
        assert!(matches!(
            users::get(tx, id).await,
            Err(StoreError::NotFound)
        ));
    }
    hit("users::create");
    hit("users::get");
    let chain = audit::list(tx, 1, 1_000).await.unwrap();
    assert!(chain.iter().all(|e| e.project_id == w.a1));
    assert_eq!(chain.len(), w.principals_in(w.a1).len());
    audit::verify_chain(&chain).unwrap();
    hit("audit::append");
    hit("audit::list");
    let pending = outbox::list_undelivered(tx, 1_000).await.unwrap();
    assert!(pending.iter().all(|e| e.project_id == w.a1.uuid()));
    assert_eq!(pending.len(), w.principals_in(w.a1).len());
    hit("outbox::enqueue");
    hit("outbox::list_undelivered");
    let foreign_jobs: Vec<Uuid> =
        sqlx::query_scalar("SELECT id FROM auvora.jobs WHERE project_id = $1")
            .bind(w.b1.uuid())
            .fetch_all(platform)
            .await
            .unwrap();
    for id in foreign_jobs {
        assert!(matches!(jobs::get(tx, id).await, Err(StoreError::NotFound)));
    }
    hit("jobs::enqueue");
    hit("jobs::get");
}

/// P2: writes aimed at B1 rows affect nothing, while the same writes on A1 work.
async fn check_writes(tx: &mut TenantTx, w: &World) {
    for id in w.principals_in(w.b1) {
        assert!(matches!(
            principals::set_status(tx, id, "suspended").await,
            Err(StoreError::NotFound)
        ));
        assert!(matches!(
            principals::delete(tx, id).await,
            Err(StoreError::NotFound)
        ));
    }
    for (id, ..) in w.identifiers_in(w.b1) {
        assert!(matches!(
            identifiers::delete(tx, id).await,
            Err(StoreError::NotFound)
        ));
    }
    for id in w.principals_in(w.b1) {
        let result = users::set_public_metadata(tx, id, &json!({"x": 1})).await;
        assert!(matches!(result, Err(StoreError::NotFound)));
    }
    if let Some(user) = principals::list(tx)
        .await
        .unwrap()
        .iter()
        .find(|p| p.kind == PrincipalKind::User)
    {
        users::set_public_metadata(tx, user.id.uuid(), &json!({"plan": "pro"}))
            .await
            .unwrap();
        assert_eq!(
            users::get(tx, user.id.uuid())
                .await
                .unwrap()
                .public_metadata,
            json!({"plan": "pro"})
        );
        hit("users::set_public_metadata");
    }
    // …while the same operations on A1's own rows work.
    let own = w.principals_in(w.a1);
    principals::set_status(tx, own[0], "suspended")
        .await
        .unwrap();
    hit("principals::set_status");
    if let Some((id, ..)) = w.identifiers_in(w.a1).first() {
        identifiers::delete(tx, *id).await.unwrap();
        hit("identifiers::delete");
    }
    principals::delete(tx, *own.last().unwrap()).await.unwrap();
    hit("principals::delete");
}

/// P3: a raw INSERT carrying another scope's tenancy keys fails `WITH CHECK`, for every table.
#[test]
fn p3_insert_with_foreign_keys_is_rejected_for_every_table() {
    let rt = runtime();
    rt.block_on(async {
        let (store, platform) = (store().await, platform_pool().await);
        let spec = ProjectSpec { kinds: vec![PrincipalKind::User], values: vec![1] };
        let w = build_world(&store, [&spec, &spec, &spec]).await;
        let app = raw_app_pool(1).await;

        for (table, level) in tenant_tables(&platform).await {
            let mut tx = app.begin().await.unwrap();
            set_scope(&mut tx, Some(w.a), Some(w.a1)).await;
            let has_id: bool = sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM pg_attribute WHERE attrelid = ('auvora.' || $1)::regclass \
                 AND attname = 'id' AND NOT attisdropped)",
            )
            .bind(&table)
            .fetch_one(&platform)
            .await
            .unwrap();
            let mut overrides = serde_json::Map::new();
            if table == "tenants" {
                overrides.insert("id".into(), json!(w.b.uuid().to_string()));
            } else {
                if has_id {
                    overrides.insert("id".into(), json!(Uuid::now_v7_for_tests().to_string()));
                }
                overrides.insert("tenant_id".into(), json!(w.b.uuid().to_string()));
                if level == "project" {
                    overrides.insert("project_id".into(), json!(w.b1.uuid().to_string()));
                }
            }
            let override_json = serde_json::Value::Object(overrides).to_string();
            let columns: Vec<String> = sqlx::query_scalar(
                "SELECT quote_ident(attname) FROM pg_attribute WHERE attrelid = ('auvora.' || $1)::regclass \
                 AND attnum > 0 AND NOT attisdropped AND attgenerated = '' ORDER BY attnum",
            )
            .bind(&table)
            .fetch_all(&mut *tx)
            .await
            .unwrap();
            let cols = columns.join(", ");
            // Copy one visible A1 row with the tenancy keys swapped to B/B1.
            let sql = format!(
                "INSERT INTO auvora.{table} ({cols}) SELECT {cols} FROM jsonb_populate_record(NULL::auvora.{table}, \
                 (SELECT to_jsonb(src) FROM auvora.{table} src LIMIT 1) || $1::jsonb)"
            );
            let err = sqlx::query(AssertSqlSafe(sql)).bind(override_json).execute(&mut *tx).await.unwrap_err();
            assert!(
                err.to_string().contains("row-level security"),
                "{table}: expected an RLS violation, got: {err}"
            );
        }
    });
}

/// P4: without context, every tenant table errors; it never returns rows.
#[test]
fn p4_missing_context_errors_on_every_table() {
    let rt = runtime();
    rt.block_on(async {
        let (store, platform) = (store().await, platform_pool().await);
        let spec = ProjectSpec {
            kinds: vec![PrincipalKind::User],
            values: vec![1],
        };
        let w = build_world(&store, [&spec, &spec, &spec]).await;
        let app = raw_app_pool(1).await;

        // First a scoped transaction on the same (single) connection, so the
        // settings exist in the session and revert to '' afterwards.
        let mut tx = app.begin().await.unwrap();
        set_scope(&mut tx, Some(w.a), Some(w.a1)).await;
        tx.commit().await.unwrap();

        for (table, _) in tenant_tables(&platform).await {
            let sql = format!("SELECT count(*) FROM auvora.{table}");
            let result = sqlx::query_scalar::<_, i64>(AssertSqlSafe(sql))
                .fetch_one(&app)
                .await;
            assert!(
                result.is_err(),
                "{table}: query without context returned {result:?}"
            );
        }
    });
}

/// P5: for every catalog table, what scope A1 sees equals the A1 slice of
/// the ground truth, for reads and for a no-op UPDATE.
#[test]
fn p5_catalog_sweep_matches_ground_truth() {
    let rt = runtime();
    rt.block_on(async {
        let (store, platform) = (store().await, platform_pool().await);
        let spec = ProjectSpec {
            kinds: vec![PrincipalKind::User, PrincipalKind::Agent],
            values: vec![1, 2],
        };
        let w = build_world(&store, [&spec, &spec, &spec]).await;
        let app = raw_app_pool(1).await;

        for (table, level) in tenant_tables(&platform).await {
            let (truth_sql, key) = match level {
                "project" => (
                    format!("SELECT count(*) FROM auvora.{table} WHERE project_id = $1"),
                    w.a1.uuid(),
                ),
                _ => (
                    format!("SELECT count(*) FROM auvora.{table} WHERE tenant_id = $1"),
                    w.a.uuid(),
                ),
            };
            let truth: i64 = sqlx::query_scalar(AssertSqlSafe(truth_sql))
                .bind(key)
                .fetch_one(&platform)
                .await
                .unwrap();

            let mut tx = app.begin().await.unwrap();
            set_scope(&mut tx, Some(w.a), Some(w.a1)).await;
            let seen: i64 = sqlx::query_scalar(AssertSqlSafe(format!(
                "SELECT count(*) FROM auvora.{table}"
            )))
            .fetch_one(&mut *tx)
            .await
            .unwrap();
            // A no-op UPDATE on the first writable column; append-only tables refuse UPDATE.
            let column: String = sqlx::query_scalar(
                "SELECT quote_ident(attname) FROM pg_attribute WHERE attrelid = ('auvora.' || $1)::regclass \
                 AND attnum > 0 AND NOT attisdropped AND attgenerated = '' ORDER BY attnum LIMIT 1",
            )
            .bind(&table)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
            let can_update: bool = sqlx::query_scalar("SELECT has_table_privilege('auvora.' || $1, 'UPDATE')")
                .bind(&table)
                .fetch_one(&mut *tx)
                .await
                .unwrap();
            let touched = if can_update {
                sqlx::query(AssertSqlSafe(format!("UPDATE auvora.{table} SET {column} = {column}")))
                    .execute(&mut *tx)
                    .await
                    .unwrap()
                    .rows_affected()
            } else {
                u64::try_from(truth).unwrap()
            };
            tx.rollback().await.unwrap();

            assert!(truth > 0, "{table}: fixture produced no A1 rows");
            assert_eq!(
                seen, truth,
                "{table}: visible rows differ from ground truth"
            );
            assert_eq!(
                i64::try_from(touched).unwrap(),
                truth,
                "{table}: UPDATE touched other scopes"
            );
        }
    });
}

/// P6: identifier uniqueness is per project, never global (ADR-0075).
#[test]
fn p6_uniqueness_is_scoped_to_the_project() {
    let rt = runtime();
    rt.block_on(async {
        let store = store().await;
        let spec = ProjectSpec {
            kinds: vec![PrincipalKind::User],
            values: vec![],
        };
        let w = build_world(&store, [&spec, &spec, &spec]).await;
        let value = [7u8; 32];

        for project in [w.a1, w.b1] {
            let mut tx = store.begin(&w.scope(project)).await.unwrap();
            let new = NewIdentifier {
                id: new_id(Kind::Identifier),
                principal_id: w.principals_in(project)[0],
                kind: "email",
                value_normalized_bidx: &value,
                value_display: None,
            };
            identifiers::create(&mut tx, &new)
                .await
                .expect("same value in another project is allowed");
            tx.commit().await.unwrap();
        }

        let mut tx = store.begin(&w.scope(w.a1)).await.unwrap();
        let duplicate = NewIdentifier {
            id: new_id(Kind::Identifier),
            principal_id: w.principals_in(w.a1)[0],
            kind: "email",
            value_normalized_bidx: &value,
            value_display: None,
        };
        let err = identifiers::create(&mut tx, &duplicate).await.unwrap_err();
        assert!(
            matches!(err, StoreError::Conflict { .. }),
            "expected conflict, got {err}"
        );
    });
}

/// P7: a child row can't reference a parent in another project.
#[test]
fn p7_cross_project_foreign_keys_are_impossible() {
    let rt = runtime();
    rt.block_on(async {
        let store = store().await;
        let spec = ProjectSpec {
            kinds: vec![PrincipalKind::User],
            values: vec![],
        };
        let w = build_world(&store, [&spec, &spec, &spec]).await;

        for foreign in [w.principals_in(w.b1)[0], w.principals_in(w.a2)[0]] {
            let mut tx = store.begin(&w.scope(w.a1)).await.unwrap();
            let new = NewIdentifier {
                id: new_id(Kind::Identifier),
                principal_id: foreign,
                kind: "username",
                value_normalized_bidx: &[9u8; 32],
                value_display: None,
            };
            let err = identifiers::create(&mut tx, &new).await.unwrap_err();
            assert!(
                err.to_string().contains("foreign key"),
                "expected FK violation, got {err}"
            );
        }
    });
}

/// P9: interleaving scoped and unscoped transactions on ONE pooled connection
/// never leaks context.
#[test]
fn p9_pooled_connection_never_leaks_context() {
    let rt = runtime();
    rt.block_on(async {
        let store = store().await;
        let spec = ProjectSpec {
            kinds: vec![PrincipalKind::User, PrincipalKind::User],
            values: vec![],
        };
        let w = build_world(&store, [&spec, &spec, &spec]).await;
        let app = raw_app_pool(1).await;
        let choices = [Some(w.a1), Some(w.b1), None];

        for i in 0..1_000usize {
            let choice = choices[(i * 7 + i / 3) % choices.len()];
            let mut tx = app.begin().await.unwrap();
            if let Some(project) = choice {
                let tenant = if project == w.b1 { w.b } else { w.a };
                set_scope(&mut tx, Some(tenant), Some(project)).await;
                let seen: Vec<Uuid> =
                    sqlx::query_scalar("SELECT project_id FROM auvora.principals")
                        .fetch_all(&mut *tx)
                        .await
                        .unwrap();
                assert!(
                    seen.iter().all(|p| *p == project.uuid()),
                    "iteration {i}: leaked rows"
                );
                assert_eq!(seen.len(), 2);
                tx.commit().await.unwrap();
            } else {
                let result =
                    sqlx::query_scalar::<_, Uuid>("SELECT project_id FROM auvora.principals")
                        .fetch_all(&mut *tx)
                        .await;
                assert!(
                    result.is_err(),
                    "iteration {i}: unscoped read returned {result:?}"
                );
                tx.rollback().await.unwrap();
            }
        }
    });
}

/// P10: a tenant-level scope sees its tenant's projects but no project data.
#[test]
fn p10_tenant_scope_sees_projects_but_no_project_data() {
    let rt = runtime();
    rt.block_on(async {
        let store = store().await;
        let spec = ProjectSpec {
            kinds: vec![PrincipalKind::User],
            values: vec![1],
        };
        let w = build_world(&store, [&spec, &spec, &spec]).await;

        let mut tx = store
            .begin(&TenantScope::tenant(w.a).unwrap())
            .await
            .unwrap();
        let ids: HashSet<Id> = projects::list(&mut tx)
            .await
            .unwrap()
            .into_iter()
            .map(|p| p.id)
            .collect();
        assert_eq!(ids, HashSet::from([w.a1, w.a2]));
        assert!(matches!(
            principals::list(&mut tx).await,
            Err(StoreError::ProjectScopeRequired)
        ));
        tx.rollback().await.unwrap();

        // The database refuses too, independently of the Rust check.
        let app = raw_app_pool(1).await;
        let mut tx = app.begin().await.unwrap();
        set_scope(&mut tx, Some(w.a), None).await;
        let result = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM auvora.principals")
            .fetch_one(&mut *tx)
            .await;
        assert!(
            result.is_err(),
            "project table readable under tenant scope: {result:?}"
        );
    });
}

/// P11: catalog lint L1–L8 (tenancy-rls.md §4.2).
#[test]
#[allow(clippy::too_many_lines)] // One linear checklist mirroring L1–L8 reads best in one place.
fn p11_catalog_lint() {
    let rt = runtime();
    rt.block_on(async {
        let platform = platform_pool().await;
        let mut problems = vec![];

        for (table, level) in tenant_tables(&platform).await {
            // L1: RLS enabled and forced.
            let (enabled, forced): (bool, bool) = sqlx::query_as(
                "SELECT relrowsecurity, relforcerowsecurity FROM pg_class WHERE oid = ('auvora.' || $1)::regclass",
            )
            .bind(&table)
            .fetch_one(&platform)
            .await
            .unwrap();
            if !(enabled && forced) {
                problems.push(format!("L1 {table}: RLS not enabled and forced"));
            }

            // L2: exactly one policy, `tenant_isolation`, with the template expression.
            let policies: Vec<(String, Option<String>, Option<String>)> = sqlx::query_as(
                "SELECT policyname::text, qual, with_check FROM pg_policies WHERE schemaname = 'auvora' AND tablename = $1",
            )
            .bind(&table)
            .fetch_all(&platform)
            .await
            .unwrap();
            let expected_fragments: &[&str] = if level == "project" {
                &["project_id = ( SELECT auvora.ctx_project_id()", "tenant_id = ( SELECT auvora.ctx_tenant_id()"]
            } else {
                &["tenant_id = ( SELECT auvora.ctx_tenant_id()"]
            };
            match policies.as_slice() {
                [(name, Some(qual), Some(check))] if name == "tenant_isolation" => {
                    for fragment in expected_fragments {
                        if !qual.contains(fragment) || !check.contains(fragment) {
                            problems.push(format!("L2 {table}: policy lacks `{fragment}`"));
                        }
                    }
                    if level == "tenant" && qual.contains("project_id") {
                        problems.push(format!("L2 {table}: tenant table policy mentions project_id"));
                    }
                }
                other => problems.push(format!("L2 {table}: expected one tenant_isolation policy, found {other:?}")),
            }

            // L3: tenancy columns are NOT NULL uuids.
            let needed: &[&str] = if level == "project" { &["tenant_id", "project_id"] } else { &["tenant_id"] };
            for column in needed {
                let ok: Option<bool> = sqlx::query_scalar(
                    "SELECT attnotnull AND atttypid = 'uuid'::regtype FROM pg_attribute \
                     WHERE attrelid = ('auvora.' || $1)::regclass AND attname = $2 AND NOT attisdropped",
                )
                .bind(&table)
                .bind(column)
                .fetch_optional(&platform)
                .await
                .unwrap();
                if ok != Some(true) {
                    problems.push(format!("L3 {table}: `{column}` must be uuid NOT NULL"));
                }
            }

            // L4: unique indexes (other than the `id` PK) lead with the tenancy key.
            let lead = if level == "project" { "project_id" } else { "tenant_id" };
            let bad: Vec<String> = sqlx::query_scalar(
                "SELECT i.relname::text FROM pg_index x JOIN pg_class i ON i.oid = x.indexrelid \
                 JOIN pg_attribute a ON a.attrelid = x.indrelid AND a.attnum = x.indkey[0] \
                 WHERE x.indrelid = ('auvora.' || $1)::regclass AND x.indisunique \
                   AND NOT (x.indisprimary AND a.attname = 'id') AND a.attname <> $2",
            )
            .bind(&table)
            .bind(lead)
            .fetch_all(&platform)
            .await
            .unwrap();
            for index in bad {
                problems.push(format!("L4 {table}: unique index `{index}` doesn't lead with {lead}"));
            }

            // L8: every column carries a PII tag (ADR-0072).
            let untagged: Vec<String> = sqlx::query_scalar(
                "SELECT attname::text FROM pg_attribute WHERE attrelid = ('auvora.' || $1)::regclass AND attnum > 0 \
                 AND NOT attisdropped AND coalesce(col_description(attrelid, attnum), '') NOT LIKE 'pii=%'",
            )
            .bind(&table)
            .fetch_all(&platform)
            .await
            .unwrap();
            for column in untagged {
                problems.push(format!("L8 {table}.{column}: missing `pii=` comment"));
            }
        }

        // L5: views run as the invoker; no materialized views.
        let unsafe_views: Vec<String> = sqlx::query_scalar(
            "SELECT c.relname::text FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace \
             WHERE n.nspname = 'auvora' AND (c.relkind = 'm' OR (c.relkind = 'v' AND \
               NOT coalesce('security_invoker=true' = ANY (c.reloptions), false)))",
        )
        .fetch_all(&platform)
        .await
        .unwrap();
        problems.extend(unsafe_views.into_iter().map(|v| format!("L5 {v}: unsafe view")));

        // L6: SECURITY DEFINER functions are exactly the allowlist.
        let definers: Vec<String> = sqlx::query_scalar(
            "SELECT p.proname::text FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace \
             WHERE n.nspname = 'auvora' AND p.prosecdef ORDER BY 1",
        )
        .fetch_all(&platform)
        .await
        .unwrap();
        if definers != DEFINER_FUNCTIONS {
            problems.push(format!("L6: SECURITY DEFINER functions {definers:?} != allowlist {DEFINER_FUNCTIONS:?}"));
        }

        // L7: the app role has no privileges on platform tables.
        for table in PLATFORM_TABLES {
            let granted: bool = sqlx::query_scalar(
                "SELECT has_table_privilege('auvora_app', 'auvora.' || $1, 'SELECT,INSERT,UPDATE,DELETE')",
            )
            .bind(table)
            .fetch_one(&platform)
            .await
            .unwrap();
            if granted {
                problems.push(format!("L7 {table}: auvora_app has privileges on a platform table"));
            }
        }

        // L7 (partitions): direct partition access would bypass the parent's RLS.
        let open_partitions: Vec<String> = sqlx::query_scalar(
            "SELECT c.relname::text FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace \
             WHERE n.nspname = 'auvora' AND c.relispartition \
               AND has_table_privilege('auvora_app', c.oid, 'SELECT,INSERT,UPDATE,DELETE')",
        )
        .fetch_all(&platform)
        .await
        .unwrap();
        problems.extend(open_partitions.into_iter().map(|t| format!("L7 {t}: auvora_app can access a partition")));

        assert!(problems.is_empty(), "catalog lint failed:\n{}", problems.join("\n"));
    });
}

/// Boot check (tenancy-rls.md §2 rule 1): only `auvora_app` may run the store.
#[test]
fn store_refuses_roles_that_can_bypass_rls() {
    let rt = runtime();
    rt.block_on(async {
        let fx = fixture();
        for role in ["postgres", "auvora_platform", "auvora_owner"] {
            let result = Store::connect(&fx.url(role), 1).await;
            assert!(
                matches!(result, Err(StoreError::UnsafeRole(_))),
                "{role} was accepted"
            );
        }
        assert!(Store::connect(&fx.url("auvora_app"), 1).await.is_ok());
    });
}

/// P8: every `PlatformScope::acquire` writes exactly one platform audit row,
/// and platform scope can claim jobs across projects.
#[test]
fn p8_platform_scope_is_audited_and_claims_across_projects() {
    let rt = runtime();
    rt.block_on(async {
        let store = store().await;
        let platform = platform_pool().await;
        let platform_store = PlatformStore::connect(&fixture().url("auvora_platform"), 2)
            .await
            .unwrap();
        let spec = ProjectSpec {
            kinds: vec![PrincipalKind::User],
            values: vec![],
        };
        let w = build_world(&store, [&spec, &spec, &spec]).await;

        // A job that is already due, in project B1.
        let due = new_id(Kind::Job);
        let mut tx = store.begin(&w.scope(w.b1)).await.unwrap();
        jobs::enqueue(&mut tx, due, "test.due", &json!({}), 1_000_000)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let ticket = format!("TICKET-{}", new_id(Kind::Event));
        let count = |ticket: String| {
            let platform = platform.clone();
            async move {
                sqlx::query_scalar::<_, i64>(
                    "SELECT count(*) FROM auvora.platform_audit_events WHERE detail = $1",
                )
                .bind(ticket)
                .fetch_one(&platform)
                .await
                .unwrap()
            }
        };
        assert_eq!(count(ticket.clone()).await, 0);

        let actor = ActorRef {
            kind: "worker",
            id: "test-worker".into(),
        };
        let mut ptx = PlatformScope::acquire(
            PlatformReason::PlatformAdminRead {
                ticket: ticket.clone(),
            },
            actor,
            &platform_store,
            &SystemClock,
        )
        .await
        .unwrap();
        let claimed = platform::claim_jobs(&mut ptx, "test-worker", 10, 60_000_000, &SystemClock)
            .await
            .unwrap();
        ptx.commit().await.unwrap();

        assert_eq!(
            count(ticket).await,
            1,
            "acquire must write exactly one platform audit row"
        );
        assert!(
            claimed
                .iter()
                .any(|j| j.id == due.uuid() && j.project_id == w.b1.uuid())
        );
        let state: String = sqlx::query_scalar("SELECT state FROM auvora.jobs WHERE id = $1")
            .bind(due.uuid())
            .fetch_one(&platform)
            .await
            .unwrap();
        assert_eq!(state, "running");

        // The app role can't read the platform chain at all.
        let app = raw_app_pool(1).await;
        let denied =
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM auvora.platform_audit_events")
                .fetch_one(&app)
                .await;
        assert!(denied.is_err(), "auvora_app read the platform audit chain");
    });
}

/// ADR-0039: the audit chain is gapless and verifiable, tampering is detected,
/// and stored events can't be changed by any role.
#[test]
fn audit_chain_is_verifiable_and_append_only() {
    let rt = runtime();
    rt.block_on(async {
        let store = store().await;
        let spec = ProjectSpec {
            kinds: vec![PrincipalKind::User; 5],
            values: vec![],
        };
        let w = build_world(&store, [&spec, &spec, &spec]).await;

        let mut tx = store.begin(&w.scope(w.a1)).await.unwrap();
        let chain = audit::list(&mut tx, 1, 100).await.unwrap();
        tx.rollback().await.unwrap();
        assert_eq!(
            chain.iter().map(|e| e.seq).collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
        audit::verify_chain(&chain).unwrap();

        let mut tampered = chain.clone();
        tampered[2].action = "principal.deleted".into();
        assert_eq!(
            audit::verify_chain(&tampered),
            Err(ChainError::HashMismatch(3))
        );
        let mut gapped = chain.clone();
        gapped.remove(1);
        assert_eq!(audit::verify_chain(&gapped), Err(ChainError::Gap(2)));

        // Floats can't be canonicalized, so they are refused before storage.
        let mut tx = store.begin(&w.scope(w.a1)).await.unwrap();
        let float_event = NewAuditEvent {
            category: "system",
            action: "x",
            outcome: "success",
            actor: json!({}),
            subject: json!({}),
            metadata: json!({"ratio": 0.5}),
        };
        assert!(matches!(
            audit::append(&mut tx, &float_event, &SystemClock).await,
            Err(StoreError::Invalid(_))
        ));
        tx.rollback().await.unwrap();

        // Append-only: app (no privilege), platform (no privilege) and owner (trigger) all fail.
        for role in ["auvora_app", "auvora_platform", "auvora_owner"] {
            let pool = PgPool::connect(&fixture().url(role)).await.unwrap();
            for sql in [
                "UPDATE auvora.audit_events SET action = 'tampered' WHERE project_id = $1",
                "DELETE FROM auvora.audit_events WHERE project_id = $1",
            ] {
                let mut tx = pool.begin().await.unwrap();
                set_scope(&mut tx, Some(w.a), Some(w.a1)).await;
                let result = sqlx::query(sql).bind(w.a1.uuid()).execute(&mut *tx).await;
                assert!(result.is_err(), "{role} could run: {sql}");
                tx.rollback().await.unwrap();
            }
        }
    });
}

// ---------------------------------------------------------------- helpers

/// Sets the transaction-local context exactly as `Store::begin` does.
async fn set_scope(tx: &mut sqlx::PgConnection, tenant: Option<Id>, project: Option<Id>) {
    let tenant = tenant.map(|t| t.uuid().to_string()).unwrap_or_default();
    let project = project.map(|p| p.uuid().to_string()).unwrap_or_default();
    sqlx::query("SELECT set_config('auvora.tenant_id', $1, true), set_config('auvora.project_id', $2, true)")
        .bind(tenant)
        .bind(project)
        .execute(tx)
        .await
        .unwrap();
}

/// Fresh random UUIDs for test rows (production IDs come from `auvora_core::id`).
trait TestUuid {
    fn now_v7_for_tests() -> Uuid;
}

impl TestUuid for Uuid {
    fn now_v7_for_tests() -> Uuid {
        Id::generate(Kind::Event, &SystemClock).expect("id").uuid()
    }
}
