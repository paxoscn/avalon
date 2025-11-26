// Agent每日统计使用示例
// 这个文件展示了如何使用AgentDailyStats功能

use chrono::{Utc, Duration};
use agent_platform::domain::entities::AgentDailyStats;
use agent_platform::domain::repositories::AgentDailyStatsRepository;
use agent_platform::domain::value_objects::{AgentId, TenantId};

/// 示例1: 记录Agent的面试统计
async fn record_interview_stats(
    repository: &dyn AgentDailyStatsRepository,
    agent_id: AgentId,
    tenant_id: TenantId,
    passed: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let today = Utc::now().date_naive();
    
    // 获取或创建今天的统计记录
    let mut stats = repository.get_or_create(&agent_id, &tenant_id, today).await?;
    
    // 增加面试次数
    stats.increment_interview();
    
    // 如果面试通过，增加通过次数
    if passed {
        stats.increment_interview_passed();
    }
    
    // 保存更新
    repository.update(&stats).await?;
    
    println!("面试统计已更新: 总面试次数={}, 通过次数={}", 
             stats.interview_count, 
             stats.interview_passed_count);
    
    Ok(())
}

/// 示例2: 记录Agent被雇佣
async fn record_employment(
    repository: &dyn AgentDailyStatsRepository,
    agent_id: AgentId,
    tenant_id: TenantId,
) -> Result<(), Box<dyn std::error::Error>> {
    let today = Utc::now().date_naive();
    let mut stats = repository.get_or_create(&agent_id, &tenant_id, today).await?;
    
    stats.increment_employment();
    repository.update(&stats).await?;
    
    println!("雇佣统计已更新: 总雇佣次数={}", stats.employment_count);
    
    Ok(())
}

/// 示例3: 记录会话和消息统计
async fn record_session_stats(
    repository: &dyn AgentDailyStatsRepository,
    agent_id: AgentId,
    tenant_id: TenantId,
    message_count: i64,
    token_count: i64,
    revenue: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let today = Utc::now().date_naive();
    let mut stats = repository.get_or_create(&agent_id, &tenant_id, today).await?;
    
    // 增加会话数
    stats.increment_session();
    
    // 添加消息和Token统计
    stats.add_messages(message_count);
    stats.add_tokens(token_count);
    
    // 添加收益
    stats.add_revenue(revenue);
    
    repository.update(&stats).await?;
    
    println!("会话统计已更新: 会话数={}, 消息数={}, Token数={}, 收益={:.4}", 
             stats.session_count,
             stats.message_count,
             stats.token_count,
             stats.revenue);
    
    Ok(())
}

/// 示例4: 查询Agent的统计数据
async fn query_agent_stats(
    repository: &dyn AgentDailyStatsRepository,
    agent_id: AgentId,
) -> Result<(), Box<dyn std::error::Error>> {
    // 查询今天的统计
    let today = Utc::now().date_naive();
    if let Some(today_stats) = repository.find_by_agent_and_date(&agent_id, today).await? {
        println!("\n=== 今日统计 ===");
        print_stats(&today_stats);
    }
    
    // 查询最近7天的统计
    let end_date = Utc::now().date_naive();
    let start_date = end_date - Duration::days(7);
    let week_stats = repository.find_by_agent_and_date_range(&agent_id, start_date, end_date).await?;
    
    if !week_stats.is_empty() {
        println!("\n=== 最近7天统计 ===");
        let total_sessions: i64 = week_stats.iter().map(|s| s.session_count).sum();
        let total_messages: i64 = week_stats.iter().map(|s| s.message_count).sum();
        let total_tokens: i64 = week_stats.iter().map(|s| s.token_count).sum();
        let total_revenue: f64 = week_stats.iter().map(|s| s.revenue).sum();
        let total_interviews: i64 = week_stats.iter().map(|s| s.interview_count).sum();
        let total_passed: i64 = week_stats.iter().map(|s| s.interview_passed_count).sum();
        let total_employments: i64 = week_stats.iter().map(|s| s.employment_count).sum();
        
        println!("总面试次数: {}", total_interviews);
        println!("面试通过次数: {}", total_passed);
        println!("总雇佣次数: {}", total_employments);
        println!("总会话数: {}", total_sessions);
        println!("总消息数: {}", total_messages);
        println!("总Token数: {}", total_tokens);
        println!("总收益: {:.4}", total_revenue);
        
        if total_interviews > 0 {
            let pass_rate = (total_passed as f64 / total_interviews as f64) * 100.0;
            println!("面试通过率: {:.2}%", pass_rate);
        }
        
        if total_sessions > 0 {
            let avg_messages = total_messages as f64 / total_sessions as f64;
            let avg_revenue = total_revenue / total_sessions as f64;
            println!("平均每会话消息数: {:.2}", avg_messages);
            println!("平均每会话收益: {:.4}", avg_revenue);
        }
    }
    
    Ok(())
}

/// 示例5: 查询租户所有Agent的统计
async fn query_tenant_stats(
    repository: &dyn AgentDailyStatsRepository,
    tenant_id: TenantId,
) -> Result<(), Box<dyn std::error::Error>> {
    let today = Utc::now().date_naive();
    let tenant_stats = repository.find_by_tenant_and_date(&tenant_id, today).await?;
    
    println!("\n=== 租户今日统计 ===");
    println!("活跃Agent数: {}", tenant_stats.len());
    
    let total_sessions: i64 = tenant_stats.iter().map(|s| s.session_count).sum();
    let total_revenue: f64 = tenant_stats.iter().map(|s| s.revenue).sum();
    let total_interviews: i64 = tenant_stats.iter().map(|s| s.interview_count).sum();
    let total_employments: i64 = tenant_stats.iter().map(|s| s.employment_count).sum();
    
    println!("总面试次数: {}", total_interviews);
    println!("总雇佣次数: {}", total_employments);
    println!("总会话数: {}", total_sessions);
    println!("总收益: {:.4}", total_revenue);
    
    // 找出收益最高的Agent
    if let Some(top_agent) = tenant_stats.iter().max_by(|a, b| {
        a.revenue.partial_cmp(&b.revenue).unwrap()
    }) {
        println!("\n收益最高的Agent:");
        print_stats(top_agent);
    }
    
    Ok(())
}

/// 辅助函数: 打印统计信息
fn print_stats(stats: &AgentDailyStats) {
    println!("日期: {}", stats.stat_date);
    println!("Agent ID: {}", stats.agent_id.0);
    println!("面试次数: {}", stats.interview_count);
    println!("面试通过次数: {}", stats.interview_passed_count);
    println!("雇佣次数: {}", stats.employment_count);
    println!("会话数: {}", stats.session_count);
    println!("消息数: {}", stats.message_count);
    println!("Token数: {}", stats.token_count);
    println!("收益: {:.4}", stats.revenue);
    println!("面试通过率: {:.2}%", stats.get_interview_pass_rate());
    println!("雇佣率: {:.2}%", stats.get_employment_rate());
    println!("平均每会话消息数: {:.2}", stats.get_average_messages_per_session());
    println!("平均每消息Token数: {:.2}", stats.get_average_tokens_per_message());
    println!("平均每会话收益: {:.4}", stats.get_average_revenue_per_session());
}

// 注意: 这是一个示例文件，不能直接运行
// 在实际使用中，你需要:
// 1. 初始化数据库连接
// 2. 创建AgentDailyStatsRepositoryImpl实例
// 3. 在适当的业务逻辑中调用这些函数
