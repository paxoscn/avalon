-- Agent每日统计表结构
-- 此文件仅用于参考，实际表由SeaORM迁移创建

CREATE TABLE IF NOT EXISTS agent_daily_stats (
    id BINARY(16) NOT NULL PRIMARY KEY,
    agent_id BINARY(16) NOT NULL,
    tenant_id BINARY(16) NOT NULL,
    stat_date DATE NOT NULL,
    interview_count BIGINT NOT NULL DEFAULT 0 COMMENT '被面试次数',
    interview_passed_count BIGINT NOT NULL DEFAULT 0 COMMENT '面试通过次数',
    employment_count BIGINT NOT NULL DEFAULT 0 COMMENT '被雇佣次数',
    session_count BIGINT NOT NULL DEFAULT 0 COMMENT '总会话数',
    message_count BIGINT NOT NULL DEFAULT 0 COMMENT '总消息数',
    token_count BIGINT NOT NULL DEFAULT 0 COMMENT '总Token消耗数',
    revenue DECIMAL(20, 6) NOT NULL DEFAULT 0.0 COMMENT '总收益',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    -- 外键约束
    CONSTRAINT fk_agent_daily_stats_agent 
        FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
    CONSTRAINT fk_agent_daily_stats_tenant 
        FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    
    -- 唯一索引：每个Agent每天只有一条记录
    UNIQUE KEY idx_agent_daily_stats_agent_date (agent_id, stat_date),
    
    -- 索引：按租户和日期查询
    KEY idx_agent_daily_stats_tenant_date (tenant_id, stat_date),
    
    -- 索引：按日期查询
    KEY idx_agent_daily_stats_stat_date (stat_date)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
COMMENT='Agent每日用量统计表';

-- 查询示例

-- 1. 查询某个Agent今天的统计
SELECT * FROM agent_daily_stats 
WHERE agent_id = ? AND stat_date = CURDATE();

-- 2. 查询某个Agent最近7天的统计
SELECT * FROM agent_daily_stats 
WHERE agent_id = ? 
  AND stat_date >= DATE_SUB(CURDATE(), INTERVAL 7 DAY)
ORDER BY stat_date DESC;

-- 3. 查询某个租户今天所有Agent的统计
SELECT * FROM agent_daily_stats 
WHERE tenant_id = ? AND stat_date = CURDATE()
ORDER BY revenue DESC;

-- 4. 查询某个租户最近30天的总收益
SELECT 
    stat_date,
    COUNT(DISTINCT agent_id) as active_agents,
    SUM(session_count) as total_sessions,
    SUM(message_count) as total_messages,
    SUM(token_count) as total_tokens,
    SUM(revenue) as total_revenue
FROM agent_daily_stats 
WHERE tenant_id = ? 
  AND stat_date >= DATE_SUB(CURDATE(), INTERVAL 30 DAY)
GROUP BY stat_date
ORDER BY stat_date DESC;

-- 5. 查询收益最高的前10个Agent（本月）
SELECT 
    agent_id,
    SUM(session_count) as total_sessions,
    SUM(message_count) as total_messages,
    SUM(revenue) as total_revenue,
    AVG(revenue / NULLIF(session_count, 0)) as avg_revenue_per_session
FROM agent_daily_stats 
WHERE stat_date >= DATE_FORMAT(CURDATE(), '%Y-%m-01')
GROUP BY agent_id
ORDER BY total_revenue DESC
LIMIT 10;

-- 6. 查询面试通过率最高的Agent（最近30天）
SELECT 
    agent_id,
    SUM(interview_count) as total_interviews,
    SUM(interview_passed_count) as total_passed,
    SUM(employment_count) as total_employments,
    (SUM(interview_passed_count) * 100.0 / NULLIF(SUM(interview_count), 0)) as pass_rate,
    (SUM(employment_count) * 100.0 / NULLIF(SUM(interview_passed_count), 0)) as employment_rate
FROM agent_daily_stats 
WHERE stat_date >= DATE_SUB(CURDATE(), INTERVAL 30 DAY)
GROUP BY agent_id
HAVING total_interviews >= 10  -- 至少有10次面试
ORDER BY pass_rate DESC
LIMIT 10;

-- 7. 按日期聚合租户的统计趋势
SELECT 
    stat_date,
    COUNT(DISTINCT agent_id) as active_agents,
    SUM(interview_count) as daily_interviews,
    SUM(interview_passed_count) as daily_passed,
    SUM(employment_count) as daily_employments,
    SUM(session_count) as daily_sessions,
    SUM(message_count) as daily_messages,
    SUM(token_count) as daily_tokens,
    SUM(revenue) as daily_revenue,
    AVG(revenue / NULLIF(session_count, 0)) as avg_revenue_per_session
FROM agent_daily_stats 
WHERE tenant_id = ? 
  AND stat_date >= DATE_SUB(CURDATE(), INTERVAL 90 DAY)
GROUP BY stat_date
ORDER BY stat_date DESC;

-- 8. 清理90天前的历史数据（可选的维护操作）
-- DELETE FROM agent_daily_stats 
-- WHERE stat_date < DATE_SUB(CURDATE(), INTERVAL 90 DAY);
