use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    email: String,
    created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Product {
    id: String,
    name: String,
    price: f64,
    stock: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== WAE Database 基础示例 ===\n");

    println!("1. 初始化模拟数据库:");
    let mut users = vec![
        User {
            id: "user-001".to_string(),
            name: "张三".to_string(),
            email: "zhangsan@example.com".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        },
        User {
            id: "user-002".to_string(),
            name: "李四".to_string(),
            email: "lisi@example.com".to_string(),
            created_at: "2024-01-02T00:00:00Z".to_string(),
        },
    ];
    let products = vec![
        Product { id: "prod-001".to_string(), name: "笔记本电脑".to_string(), price: 5999.0, stock: 50 },
        Product { id: "prod-002".to_string(), name: "无线鼠标".to_string(), price: 99.0, stock: 200 },
    ];
    println!("   模拟数据库初始化完成\n");

    println!("2. 查询所有用户 (SELECT):");
    println!("   找到 {} 个用户:", users.len());
    for user in &users {
        println!("   - {} ({})", user.name, user.email);
    }
    println!();

    println!("3. 查询单个用户 (WHERE):");
    let user_id = "user-001";
    if let Some(user) = users.iter().find(|u| u.id == user_id) {
        println!("   找到用户:");
        println!("     ID: {}", user.id);
        println!("     姓名: {}", user.name);
        println!("     邮箱: {}", user.email);
        println!("     创建时间: {}", user.created_at);
    }
    println!();

    println!("4. 创建新用户 (INSERT):");
    let new_user = User {
        id: "user-003".to_string(),
        name: "王五".to_string(),
        email: "wangwu@example.com".to_string(),
        created_at: "2024-01-03T00:00:00Z".to_string(),
    };
    println!("   创建用户: {} ({})", new_user.name, new_user.email);
    users.push(new_user);
    println!("   当前用户数量: {}\n", users.len());

    println!("5. 更新用户 (UPDATE):");
    let update_id = "user-001";
    if let Some(user) = users.iter_mut().find(|u| u.id == update_id) {
        user.name = "张三（已更新）".to_string();
        user.email = "zhangsan_updated@example.com".to_string();
        println!("   更新用户:");
        println!("     新姓名: {}", user.name);
        println!("     新邮箱: {}", user.email);
    }
    println!();

    println!("6. 查询所有产品:");
    println!("   找到 {} 个产品:", products.len());
    for product in &products {
        println!("   - {}: ¥{} (库存: {})", product.name, product.price, product.stock);
    }
    println!();

    println!("7. 删除用户 (DELETE):");
    let delete_id = "user-002";
    let initial_count = users.len();
    users.retain(|u| u.id != delete_id);
    println!("   删除用户 ID: {} (删除前: {}, 删除后: {})", delete_id, initial_count, users.len());
    println!();

    println!("=== 示例总结 ===");
    println!("本示例展示了基本的数据库 CRUD 操作:");
    println!("  - CREATE (INSERT): 创建新记录");
    println!("  - READ (SELECT): 查询记录");
    println!("  - UPDATE: 更新记录");
    println!("  - DELETE: 删除记录");
    println!();
    println!("注意: 本示例使用内存数据结构模拟数据库");
    println!("实际使用时需要配置 Turso/PostgreSQL/MySQL 连接");
    println!();

    Ok(())
}
