//! RemDB Milvus 兼容 API 完整示例
//!
//! 本示例演示了如何使用 RemDB 的 Milvus 兼容 RESTful API 进行向量数据库操作。
//! 场景：电商商品向量检索系统 —— 管理商品数据并支持相似商品搜索。
//!
//! 运行方式：
//!   cargo run --example milvus_demo --release
//!
//! 注意：首次运行会编译所有依赖，需要较长时间。后续运行使用缓存。
//! 本示例使用 warp::test 模拟 HTTP 请求，无需启动真实服务器。

use std::sync::Arc;

use remdb::memory::allocator;
use serde_json::json;
use warp::Filter;

use remdb_server::milvus::catalog::MilvusCatalog;
use remdb_server::milvus::handler;
use remdb_server::config::RuntimeConfig;

// ── 辅助函数 ──

/// 打印带分隔线的标题
fn print_title(title: &str) {
    println!("\n{}", "=".repeat(72));
    println!("  {}", title);
    println!("{}", "=".repeat(72));
}

/// 打印 JSON 响应（格式化）
fn print_response(label: &str, body: &[u8]) {
    println!("  {}:", label);
    match serde_json::from_slice::<serde_json::Value>(body) {
        Ok(json) => println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default()),
        Err(_) => println!("    (raw) {}", String::from_utf8_lossy(body)),
    }
    println!();
}

/// 打印操作结果
fn print_result(label: &str, body: &[u8]) {
    let ok = serde_json::from_slice::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("code").and_then(|c| c.as_i64()))
        == Some(0);
    if ok {
        println!("  {} ✅", label);
    } else {
        let msg = serde_json::from_slice::<serde_json::Value>(body)
            .ok()
            .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown error".to_string());
        println!("  {} ❌ {}", label, msg);
    }
    println!();
}

// ── 主函数 ──

#[tokio::main]
async fn main() {
    // 初始化 tracing 日志（仅显示 warn 及以上级别，避免干扰示例输出）
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();

    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║         RemDB Milvus 兼容 API 完整示例                              ║");
    println!("║         场景：电商商品向量检索系统                                  ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");

    // ═══════════════════════════════════════════════════════════════════════════════
    // 第一步：初始化全局内存分配器
    // ═══════════════════════════════════════════════════════════════════════════════
    print_title("第一步：初始化全局内存分配器");

    let total_memory = 1024 * 1024 * 200; // 200 MB
    let memory_vec: Vec<u8> = Vec::with_capacity(total_memory);
    let memory_ptr = memory_vec.as_ptr() as *mut u8;
    std::mem::forget(memory_vec); // 防止 Vec 析构时释放内存

    unsafe {
        allocator::init_global_allocator(memory_ptr, total_memory)
            .expect("初始化全局内存分配器失败");
    }
    println!("  ✅ 全局内存分配器初始化完成（200 MB）\n");

    // ═══════════════════════════════════════════════════════════════════════════════
    // 第二步：初始化数据库
    // ═══════════════════════════════════════════════════════════════════════════════
    print_title("第二步：初始化数据库");

    let config = RuntimeConfig {
        snapshot_dir: Some("./demo_snapshots".to_string()),
        full_image: None,
        total_memory,
        default_max_records: 10000,
        low_power_mode_supported: false,
        low_power_max_records: None,
        log_path: Some("./demo_logs".to_string()),
        log_file_name: "./demo_logs/demo.log".to_string(),
        snapshot_interval: None,
        snapshot_type: None,
        max_incremental_snapshots: None,
        debug_mode: false,
        jdbc: remdb_server::config::JdbcConfig {
            port: Some(16666),
            enabled: Some(false), // 不启动 JDBC 服务
            max_connections: Some(10),
            timeout: Some(30),
            auth_enabled: Some(false),
            username: None,
            password_hash: None,
        },
        pubsub: remdb_server::config::PubSubConfig::default(),
        ha: remdb_server::config::HaConfig::default(),
        wal: remdb_server::config::WALConfig::default(),
        ddl_path: None,
    };

    let ctx = remdb_server::context::AppContextBuilder::new()
        .with_config(config)
        .with_tables(vec![])
        .build()
        .expect("初始化数据库失败");

    println!("  ✅ 数据库初始化完成\n");

    // ═══════════════════════════════════════════════════════════════════════════════
    // 第三步：初始化 Milvus 兼容层
    // ═══════════════════════════════════════════════════════════════════════════════
    print_title("第三步：初始化 Milvus 兼容层");

    // 从 AppContext 获取数据库实例的引用（AppContext.db 是 pub 字段）
    let db: Arc<std::sync::Mutex<&'static mut remdb::RemDb>> = ctx.db.clone();

    // 创建 Milvus 目录
    let catalog = Arc::new(MilvusCatalog::new(db.clone()));

    // 初始化目录（创建 _milvus_catalog 系统表并刷新缓存）
    catalog.init().await.expect("初始化 Milvus 目录失败");
    println!("  ✅ Milvus 目录初始化完成\n");

    // ═══════════════════════════════════════════════════════════════════════════════
    // 第四步：构建 Warp 路由
    // ═══════════════════════════════════════════════════════════════════════════════
    print_title("第四步：构建 API 路由");

    // 无认证过滤器
    let auth = warp::any()
        .and_then(|| async move { Ok::<_, warp::Rejection>(()) })
        .map(|_: ()| ())
        .untuple_one()
        .boxed();

    let catalog_filter = warp::any().map(move || catalog.clone()).boxed();

    // ── 集合路由 ──
    let create_collection = warp::path!("v2" / "vectordb" / "collections" / "create")
        .and(warp::post())
        .and(auth.clone())
        .and(catalog_filter.clone())
        .and(warp::body::json())
        .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
            handler::handle_create_collection(catalog, body).await
        });

    let drop_collection = warp::path!("v2" / "vectordb" / "collections" / "drop")
        .and(warp::post())
        .and(auth.clone())
        .and(catalog_filter.clone())
        .and(warp::body::json())
        .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
            handler::handle_drop_collection(catalog, body).await
        });

    let list_collections = warp::path!("v2" / "vectordb" / "collections" / "list")
        .and(warp::get())
        .and(auth.clone())
        .and(catalog_filter.clone())
        .and_then(|catalog: Arc<MilvusCatalog>| async move {
            handler::handle_list_collections(catalog).await
        });

    let describe_collection = warp::path!("v2" / "vectordb" / "collections" / "describe")
        .and(warp::post())
        .and(auth.clone())
        .and(catalog_filter.clone())
        .and(warp::body::json())
        .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
            handler::handle_describe_collection(catalog, body).await
        });

    let has_collection = warp::path!("v2" / "vectordb" / "collections" / "has")
        .and(warp::post())
        .and(auth.clone())
        .and(catalog_filter.clone())
        .and(warp::body::json())
        .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
            handler::handle_has_collection(catalog, body).await
        });

    // ── 实体路由 ──
    let insert = warp::path!("v2" / "vectordb" / "entities" / "insert")
        .and(warp::post())
        .and(auth.clone())
        .and(catalog_filter.clone())
        .and(warp::body::json())
        .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
            handler::handle_insert(catalog, body).await
        });

    let upsert = warp::path!("v2" / "vectordb" / "entities" / "upsert")
        .and(warp::post())
        .and(auth.clone())
        .and(catalog_filter.clone())
        .and(warp::body::json())
        .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
            handler::handle_upsert(catalog, body).await
        });

    let delete = warp::path!("v2" / "vectordb" / "entities" / "delete")
        .and(warp::post())
        .and(auth.clone())
        .and(catalog_filter.clone())
        .and(warp::body::json())
        .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
            handler::handle_delete(catalog, body).await
        });

    let get = warp::path!("v2" / "vectordb" / "entities" / "get")
        .and(warp::post())
        .and(auth.clone())
        .and(catalog_filter.clone())
        .and(warp::body::json())
        .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
            handler::handle_get(catalog, body).await
        });

    let query = warp::path!("v2" / "vectordb" / "entities" / "query")
        .and(warp::post())
        .and(auth.clone())
        .and(catalog_filter.clone())
        .and(warp::body::json())
        .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
            handler::handle_query(catalog, body).await
        });

    let search = warp::path!("v2" / "vectordb" / "entities" / "search")
        .and(warp::post())
        .and(auth.clone())
        .and(catalog_filter.clone())
        .and(warp::body::json())
        .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
            handler::handle_search(catalog, body).await
        });

    // ── 索引路由 ──
    let create_index = warp::path!("v2" / "vectordb" / "indexes" / "create")
        .and(warp::post())
        .and(auth.clone())
        .and(catalog_filter.clone())
        .and(warp::body::json())
        .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
            handler::handle_create_index(catalog, body).await
        });

    let drop_index = warp::path!("v2" / "vectordb" / "indexes" / "drop")
        .and(warp::post())
        .and(auth.clone())
        .and(catalog_filter.clone())
        .and(warp::body::json())
        .and_then(|catalog: Arc<MilvusCatalog>, body| async move {
            handler::handle_drop_index(catalog, body).await
        });

    // 合并所有路由
    let routes = create_collection
        .or(drop_collection)
        .or(list_collections)
        .or(describe_collection)
        .or(has_collection)
        .or(insert)
        .or(upsert)
        .or(delete)
        .or(get)
        .or(query)
        .or(search)
        .or(create_index)
        .or(drop_index)
        .with(warp::cors().allow_any_origin())
        .recover(handler::handle_rejection);

    println!("  ✅ API 路由构建完成（共 13 个端点）\n");

    // ═══════════════════════════════════════════════════════════════════════════════
    // 第五步：演示 API 调用
    // ═══════════════════════════════════════════════════════════════════════════════
    print_title("第五步：演示 Milvus 兼容 API 调用");
    println!("  场景：电商商品向量检索系统");
    println!("  - 创建商品集合（含 4 维向量用于相似度搜索）");
    println!("  - 插入商品数据（名称、价格、类别、特征向量）");
    println!("  - 查询和搜索商品");
    println!("  - 更新和删除商品\n");

    // ── 5.1 创建集合 ──
    print_title("5.1 POST /v2/vectordb/collections/create — 创建集合");

    let create_body = json!({
        "collectionName": "ecommerce_products",
        "description": "电商商品集合，支持向量相似度搜索",
        "schema": {
            "autoId": true,
            "description": "商品信息：名称、价格、类别、特征向量",
            "fields": [
                {
                    "name": "product_id",
                    "type": "Int64",
                    "isPrimary": true,
                    "autoId": true
                },
                {
                    "name": "name",
                    "type": "VarChar",
                    "params": { "max_length": 128 }
                },
                {
                    "name": "price",
                    "type": "Float"
                },
                {
                    "name": "category",
                    "type": "VarChar",
                    "params": { "max_length": 64 }
                },
                {
                    "name": "features",
                    "type": "FloatVector",
                    "params": { "dim": 4 }
                }
            ]
        },
        "indexParams": [{
            "fieldName": "features",
            "indexName": "idx_features",
            "metricType": "L2",
            "params": {
                "nlist": 128,
                "M": 16,
                "efConstruction": 200,
                "index_type": "HNSW"
            }
        }]
    });

    let resp = warp::test::request()
        .method("POST")
        .path("/v2/vectordb/collections/create")
        .json(&create_body)
        .reply(&routes)
        .await;

    print_result("创建集合 'ecommerce_products'", resp.body());
    print_response("响应详情", resp.body());

    // 如果集合已存在，继续演示
    let create_ok = serde_json::from_slice::<serde_json::Value>(resp.body())
        .ok()
        .and_then(|v| v.get("code").and_then(|c| c.as_i64()))
        == Some(0);
    if !create_ok {
        println!("  ⚠ 集合可能已存在，继续演示...\n");
    }

    // ── 5.2 列出集合 ──
    print_title("5.2 GET /v2/vectordb/collections/list — 列出所有集合");

    let resp = warp::test::request()
        .method("GET")
        .path("/v2/vectordb/collections/list")
        .reply(&routes)
        .await;

    print_result("列出集合", resp.body());
    print_response("响应详情", resp.body());

    // ── 5.3 描述集合 ──
    print_title("5.3 POST /v2/vectordb/collections/describe — 描述集合详情");

    let describe_body = json!({
        "collectionName": "ecommerce_products"
    });

    let resp = warp::test::request()
        .method("POST")
        .path("/v2/vectordb/collections/describe")
        .json(&describe_body)
        .reply(&routes)
        .await;

    print_result("描述集合 'ecommerce_products'", resp.body());
    print_response("响应详情", resp.body());

    // ── 5.4 检查集合是否存在 ──
    print_title("5.4 POST /v2/vectordb/collections/has — 检查集合是否存在");

    let has_body = json!({
        "collectionName": "ecommerce_products"
    });

    let resp = warp::test::request()
        .method("POST")
        .path("/v2/vectordb/collections/has")
        .json(&has_body)
        .reply(&routes)
        .await;

    print_result("检查集合 'ecommerce_products'", resp.body());
    print_response("响应详情", resp.body());

    // ── 5.5 插入实体 ──
    print_title("5.5 POST /v2/vectordb/entities/insert — 插入商品数据");

    let insert_body = json!({
        "collectionName": "ecommerce_products",
        "data": [
            {
                "name": "智能蓝牙耳机 Pro",
                "price": 299.00,
                "category": "电子产品",
                "features": [0.85, 0.12, 0.45, 0.67]
            },
            {
                "name": "轻薄笔记本 Air",
                "price": 5999.00,
                "category": "电子产品",
                "features": [0.92, 0.34, 0.15, 0.78]
            },
            {
                "name": "纯棉T恤基础款",
                "price": 59.00,
                "category": "服装",
                "features": [0.15, 0.88, 0.72, 0.23]
            },
            {
                "name": "运动跑鞋 Ultra",
                "price": 459.00,
                "category": "运动",
                "features": [0.35, 0.75, 0.62, 0.91]
            },
            {
                "name": "机械键盘 87键",
                "price": 199.00,
                "category": "电子产品",
                "features": [0.78, 0.25, 0.55, 0.44]
            },
            {
                "name": "瑜伽垫加厚防滑",
                "price": 89.00,
                "category": "运动",
                "features": [0.25, 0.65, 0.85, 0.33]
            },
            {
                "name": "真无线降噪耳机",
                "price": 899.00,
                "category": "电子产品",
                "features": [0.88, 0.08, 0.42, 0.72]
            },
            {
                "name": "休闲卫衣连帽",
                "price": 129.00,
                "category": "服装",
                "features": [0.18, 0.82, 0.78, 0.19]
            }
        ]
    });

    let resp = warp::test::request()
        .method("POST")
        .path("/v2/vectordb/entities/insert")
        .json(&insert_body)
        .reply(&routes)
        .await;

    print_result("插入 8 个商品", resp.body());
    print_response("响应详情", resp.body());

    // 提取插入的 IDs 供后续操作使用
    let inserted_ids: Vec<i64> = serde_json::from_slice::<serde_json::Value>(resp.body())
        .ok()
        .and_then(|v| v.get("data").and_then(|d| d.get("insertIds")).cloned())
        .and_then(|ids| serde_json::from_value(ids).ok())
        .unwrap_or_default();
    println!("  📝 插入的 ID 列表: {:?}\n", inserted_ids);

    // ── 5.6 根据 ID 获取实体 ──
    print_title("5.6 POST /v2/vectordb/entities/get — 根据 ID 获取商品");

    if let Some(first_id) = inserted_ids.first() {
        let get_body = json!({
            "collectionName": "ecommerce_products",
            "id": first_id,
            "outputFields": ["product_id", "name", "price", "category"]
        });

        let resp = warp::test::request()
            .method("POST")
            .path("/v2/vectordb/entities/get")
            .json(&get_body)
            .reply(&routes)
            .await;

        print_result(&format!("获取 ID={} 的商品", first_id), resp.body());
        print_response("响应详情", resp.body());
    }

    // ── 5.7 查询实体 ──
    print_title("5.7 POST /v2/vectordb/entities/query — 查询商品（电子产品）");

    let query_body = json!({
        "collectionName": "ecommerce_products",
        "filter": "category = '电子产品'",
        "outputFields": ["product_id", "name", "price", "category"],
        "limit": 10,
        "offset": 0
    });

    let resp = warp::test::request()
        .method("POST")
        .path("/v2/vectordb/entities/query")
        .json(&query_body)
        .reply(&routes)
        .await;

    print_result("查询电子产品", resp.body());
    print_response("响应详情", resp.body());

    // ── 5.8 向量搜索 ──
    print_title("5.8 POST /v2/vectordb/entities/search — 向量搜索相似商品");

    // 搜索与"智能蓝牙耳机"（特征: [0.85, 0.12, 0.45, 0.67]）相似的商品
    let search_body = json!({
        "collectionName": "ecommerce_products",
        "vector": [0.80, 0.15, 0.40, 0.70],
        "annsField": "features",
        "limit": 5,
        "offset": 0,
        "outputFields": ["product_id", "name", "price", "category"],
        "params": {
            "ef": 64,
            "nprobe": 8
        }
    });

    let resp = warp::test::request()
        .method("POST")
        .path("/v2/vectordb/entities/search")
        .json(&search_body)
        .reply(&routes)
        .await;

    print_result("向量搜索（与蓝牙耳机相似的商品）", resp.body());
    print_response("搜索结果", resp.body());

    // ── 5.9 Upsert（更新或插入） ──
    print_title("5.9 POST /v2/vectordb/entities/upsert — 更新商品价格");

    if let Some(first_id) = inserted_ids.first() {
        let upsert_body = json!({
            "collectionName": "ecommerce_products",
            "data": [
                {
                    "product_id": first_id,
                    "name": "智能蓝牙耳机 Pro",
                    "price": 249.00,
                    "category": "电子产品",
                    "features": [0.85, 0.12, 0.45, 0.67]
                }
            ]
        });

        let resp = warp::test::request()
            .method("POST")
            .path("/v2/vectordb/entities/upsert")
            .json(&upsert_body)
            .reply(&routes)
            .await;

        print_result("Upsert 更新价格", resp.body());
        print_response("Upsert 响应", resp.body());

        // 验证更新后的价格
        let get_body = json!({
            "collectionName": "ecommerce_products",
            "id": first_id,
            "outputFields": ["product_id", "name", "price"]
        });

        let resp = warp::test::request()
            .method("POST")
            .path("/v2/vectordb/entities/get")
            .json(&get_body)
            .reply(&routes)
            .await;

        println!("  📝 验证更新结果：");
        print_response("更新后查询", resp.body());
    }

    // ── 5.10 删除实体 ──
    print_title("5.10 POST /v2/vectordb/entities/delete — 删除商品");

    if let Some(last_id) = inserted_ids.last() {
        let delete_body = json!({
            "collectionName": "ecommerce_products",
            "filter": format!("id in [{}]", last_id)
        });

        let resp = warp::test::request()
            .method("POST")
            .path("/v2/vectordb/entities/delete")
            .json(&delete_body)
            .reply(&routes)
            .await;

        print_result(&format!("删除 ID={} 的商品", last_id), resp.body());
        print_response("删除响应", resp.body());
    }

    // ── 5.11 创建索引 ──
    print_title("5.11 POST /v2/vectordb/indexes/create — 创建索引");

    let create_index_body = json!({
        "collectionName": "ecommerce_products",
        "indexName": "idx_features",
        "fieldName": "features",
        "metricType": "L2",
        "params": {
            "nlist": 128,
            "M": 16,
            "efConstruction": 200
        }
    });

    let resp = warp::test::request()
        .method("POST")
        .path("/v2/vectordb/indexes/create")
        .json(&create_index_body)
        .reply(&routes)
        .await;

    print_result("创建索引 'idx_features'", resp.body());
    print_response("响应详情", resp.body());

    // ── 5.12 删除索引 ──
    print_title("5.12 POST /v2/vectordb/indexes/drop — 删除索引");

    let drop_index_body = json!({
        "collectionName": "ecommerce_products",
        "indexName": "idx_features"
    });

    let resp = warp::test::request()
        .method("POST")
        .path("/v2/vectordb/indexes/drop")
        .json(&drop_index_body)
        .reply(&routes)
        .await;

    print_result("删除索引 'idx_features'", resp.body());
    print_response("响应详情", resp.body());

    // ── 5.13 删除集合 ──
    print_title("5.13 POST /v2/vectordb/collections/drop — 删除集合（清理）");

    let drop_body = json!({
        "collectionName": "ecommerce_products"
    });

    let resp = warp::test::request()
        .method("POST")
        .path("/v2/vectordb/collections/drop")
        .json(&drop_body)
        .reply(&routes)
        .await;

    print_result("删除集合 'ecommerce_products'", resp.body());
    print_response("响应详情", resp.body());

    // ═══════════════════════════════════════════════════════════════════════════════
    // 总结
    // ═══════════════════════════════════════════════════════════════════════════════
    print_title("🎉 演示完成！");

    println!("  已演示的 API 端点：");
    println!("    ✓ POST /v2/vectordb/collections/create  - 创建集合");
    println!("    ✓ GET  /v2/vectordb/collections/list    - 列出集合");
    println!("    ✓ POST /v2/vectordb/collections/describe - 描述集合");
    println!("    ✓ POST /v2/vectordb/collections/has     - 检查集合");
    println!("    ✓ POST /v2/vectordb/entities/insert     - 插入实体");
    println!("    ✓ POST /v2/vectordb/entities/get        - 获取实体");
    println!("    ✓ POST /v2/vectordb/entities/query      - 查询实体");
    println!("    ✓ POST /v2/vectordb/entities/search     - 向量搜索");
    println!("    ✓ POST /v2/vectordb/entities/upsert     - 更新/插入");
    println!("    ✓ POST /v2/vectordb/entities/delete     - 删除实体");
    println!("    ✓ POST /v2/vectordb/indexes/create      - 创建索引");
    println!("    ✓ POST /v2/vectordb/indexes/drop        - 删除索引");
    println!("    ✓ POST /v2/vectordb/collections/drop    - 删除集合");
    println!();
    println!("  场景：电商商品向量检索系统");
    println!("  - 创建了包含 5 个字段（含 4 维向量）的商品集合");
    println!("  - 插入了 8 个商品（电子产品、服装、运动）");
    println!("  - 使用 L2 距离度量进行向量相似度搜索");
    println!("  - 演示了完整的 CRUD + 搜索生命周期");
    println!();
    println!("  API 接口兼容 Milvus v2.3+ RESTful 风格。");
    println!();
    println!("  运行方式: cargo run --example milvus_demo --release");
}