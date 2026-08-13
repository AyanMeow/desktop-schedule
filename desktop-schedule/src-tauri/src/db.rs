use chrono::NaiveDate;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// 一条日程记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: i64,
    pub group_id: Option<String>,
    pub title: String,
    pub date: String, // 'YYYY-MM-DD'
    pub time_of_day: Option<String>, // 'HH:MM'
    pub note: Option<String>,
    pub priority: i64,
    pub has_ddl: bool,
    pub ddl_at: Option<String>, // 'YYYY-MM-DD HH:MM' 或 'YYYY-MM-DD'
    pub completed: bool,
    pub completed_at: Option<String>,
    pub attachment: Option<String>, // 关联的文件/文件夹路径
    pub created_at: String,
    pub updated_at: String,
}

/// 新建日程的输入（支持跨天范围填充）
#[derive(Debug, Clone, Deserialize)]
pub struct NewSchedule {
    pub title: String,
    pub start_date: String, // 'YYYY-MM-DD'（含）
    pub end_date: String,   // 'YYYY-MM-DD'（含）
    pub time_of_day: Option<String>,
    pub note: Option<String>,
    #[serde(default)]
    pub priority: i64,
    pub ddl_at: Option<String>,
    pub attachment: Option<String>, // 关联的文件/文件夹路径
}

/// 数据库连接的线程安全封装
pub struct Db(pub Mutex<Connection>);

impl Db {
    pub fn new(conn: Connection) -> Self {
        Self(Mutex::new(conn))
    }
}

/// 打开/创建数据库并执行版本迁移
pub fn open(path: &std::path::Path) -> anyhow::Result<Db> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    ensure_schema(&conn)?;
    run_migrations(&conn)?;
    Ok(Db::new(conn))
}

// ============ 数据库版本迁移框架 ============
//
// 【如何新增一个版本】（未来更新时按此操作）：
// 1. 把 CURRENT_DB_VERSION 加 1（如 2 → 3）
// 2. 在 run_migrations 里追加一个 `if version < N` 块：
//        if version < 3 {
//            // 迁移逻辑（必须幂等：重复执行不报错）
//            version = 3;
//        }
// 3. 该块内的 SQL 操作要写成幂等（用 IF NOT EXISTS / 先检测再改）
//
// 全新用户：user_version 默认 0，从 v0 依次跑完所有迁移
// 老用户：从自己的 user_version 跑到 CURRENT，跳过已执行的
// 迁移函数必须幂等，保证重复执行安全。

/// 当前数据库版本（每次需要数据迁移的发布都要 +1）
const CURRENT_DB_VERSION: u32 = 2;

/// 建表（含最新结构）。IF NOT EXISTS 对老库无副作用。
fn ensure_schema(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS schedules (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            group_id      TEXT,
            title         TEXT NOT NULL,
            date          TEXT NOT NULL,
            time_of_day   TEXT,
            note          TEXT,
            priority      INTEGER NOT NULL DEFAULT 0,
            has_ddl       INTEGER NOT NULL DEFAULT 0,
            ddl_at        TEXT,
            completed     INTEGER NOT NULL DEFAULT 0,
            completed_at  TEXT,
            attachment    TEXT,
            created_at    TEXT NOT NULL,
            updated_at    TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_schedules_date ON schedules(date);
        CREATE INDEX IF NOT EXISTS idx_schedules_group ON schedules(group_id);

        CREATE TABLE IF NOT EXISTS encouragements (
            id       INTEGER PRIMARY KEY AUTOINCREMENT,
            text     TEXT NOT NULL,
            category TEXT,
            enabled  INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS completion_log (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            schedule_id  INTEGER,
            completed_at TEXT
        );
        ",
    )?;
    Ok(())
}

/// 执行版本迁移链。从 user_version 逐级升到 CURRENT_DB_VERSION。
fn run_migrations(conn: &Connection) -> anyhow::Result<()> {
    let mut version: u32 =
        conn.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))? as u32;

    // v0 → v1：schedules 表补 attachment 列（幂等：检测列是否存在）
    if version < 1 {
        let has_attachment: bool = conn
            .prepare("PRAGMA table_info(schedules)")?
            .query_map([], |r| r.get::<_, String>(1))?
            .filter_map(|r| r.ok())
            .any(|name| name == "attachment");
        if !has_attachment {
            conn.execute("ALTER TABLE schedules ADD COLUMN attachment TEXT", [])?;
        }
        version = 1;
    }

    // v1 → v2：鼓励语清空重建（替换为艾露猫风格文案）
    // 幂等：DELETE 后 seed，重复执行结果一致
    if version < 2 {
        conn.execute("DELETE FROM encouragements", [])?;
        seed_encouragements(conn)?;
        version = 2;
    }

    // ---- 未来迁移在此追加（示例，勿删注释）----
    // if version < 3 {
    //     // 新的迁移逻辑（幂等）
    //     version = 3;
    // }

    // 写回版本号
    if version > 0 {
        conn.execute_batch(&format!("PRAGMA user_version = {}", version))?;
    }
    Ok(())
}

/// 首次启动时填充鼓励语库
fn seed_encouragements(conn: &Connection) -> anyhow::Result<()> {
    let count: i64 =
        conn.query_row("SELECT COUNT(*) FROM encouragements", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }
    let items: &[(&str, &str)] = &[
        // 热血（8 条）
        ("老大又完成了一项喵！老大真厉害喵！", "热血"),
        ("加油哇老大，再完成几项任务咱们就是优秀猎人了喵！", "热血"),
        ("老大今天的气势，简直像吃了猫饭一样足喵！", "热血"),
        ("这项任务被老大轻松拿下喵，老大真牛逼！", "热血"),
        ("一鼓作气喵！老大离传说级猎人又近了一步！", "热血"),
        ("老大的执行力，连看板娘都竖起大拇指喵！", "热血"),
        ("又是满载而归的一天喵，老大该吃团子奖励自己了喵！", "热血"),
        ("干劲保持下去，整个公会都没有老大的对手喵！", "热血"),
        // 调皮（6 条）
        ("喵！这任务看到老大，吓得自己变成素材跑掉了喵！", "调皮"),
        ("叮！老大经验值 +1，快升级了喵要不要回去交任务？", "调皮"),
        ("任务完成喵！老大要不要奖励自己一份猫饭？", "调皮"),
        ("今日老大战斗力评测：MR 解锁级喵！", "调皮"),
        ("这效率喵，开挂了喵！", "调皮"),
        ("这速度喵，连迅龙都追不上老大！", "调皮"),
        // 暖心（7 条）
        ("老大加油！老大努力！", "暖心"),
        ("慢慢来就好喵，老大今天已经很努力了喵。", "暖心"),
        ("记得喝口水休息一下喵，老大辛苦啦。", "暖心"),
        ("别太勉强自己喵，累了就猫车回营地歇会儿吧。", "暖心"),
        ("哪怕只完成一项，老大在咱们心里也是最棒的喵。", "暖心"),
        ("深呼吸喵，不管多难的任务，咱们一起猫车喵。", "暖心"),
        ("今天的努力不会白费的喵，好好睡一觉吧老大。", "暖心"),
    ];
    let mut stmt = conn.prepare("INSERT INTO encouragements (text, category, enabled) VALUES (?1, ?2, 1)")?;
    for (text, cat) in items {
        stmt.execute(params![text, cat])?;
    }
    Ok(())
}

// ============ CRUD ============

/// 新建日程（自动按日期范围展开为多条独立记录）
/// 返回生成的所有记录 id
pub fn create_schedules(conn: &Connection, input: &NewSchedule) -> anyhow::Result<Vec<i64>> {
    let start = NaiveDate::parse_from_str(&input.start_date, "%Y-%m-%d")
        .map_err(|e| anyhow::anyhow!("start_date 格式错误: {e}"))?;
    let end = NaiveDate::parse_from_str(&input.end_date, "%Y-%m-%d")
        .map_err(|e| anyhow::anyhow!("end_date 格式错误: {e}"))?;
    if end < start {
        return Err(anyhow::anyhow!("end_date 不能早于 start_date"));
    }

    let group_id = uuid::Uuid::new_v4().to_string();
    let has_ddl = input.ddl_at.is_some();
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut ids = Vec::new();
    let tx = conn.unchecked_transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO schedules
                (group_id, title, date, time_of_day, note, priority, has_ddl, ddl_at, completed, attachment, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0, ?9, ?10, ?10)",
        )?;
        let mut cur = start;
        loop {
            stmt.execute(params![
                group_id,
                input.title,
                cur.format("%Y-%m-%d").to_string(),
                input.time_of_day,
                input.note,
                input.priority,
                has_ddl as i64,
                input.ddl_at,
                input.attachment,
                now,
            ])?;
            ids.push(tx.last_insert_rowid());
            if cur == end {
                break;
            }
            cur = cur.succ_opt().ok_or_else(|| anyhow::anyhow!("日期溢出"))?;
        }
    }
    tx.commit()?;
    Ok(ids)
}

/// 查询某日期范围内每天的日程（按 date, time_of_day, created_at 排序）
pub fn list_schedules_in_range(
    conn: &Connection,
    start: &str,
    end: &str,
) -> anyhow::Result<Vec<Schedule>> {
    let mut stmt = conn.prepare(
        "SELECT id, group_id, title, date, time_of_day, note, priority, has_ddl, ddl_at,
                completed, completed_at, attachment, created_at, updated_at
         FROM schedules
         WHERE date BETWEEN ?1 AND ?2
         ORDER BY date ASC,
                  (time_of_day IS NULL), time_of_day ASC,
                  created_at ASC",
    )?;
    let rows = stmt
        .query_map(params![start, end], row_to_schedule)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// 查询单日日程
pub fn list_schedules_on_day(conn: &Connection, date: &str) -> anyhow::Result<Vec<Schedule>> {
    list_schedules_in_range(conn, date, date)
}

/// 切换完成状态，返回新状态与是否变为完成（用于触发鼓励语）
pub fn toggle_completed(conn: &Connection, id: i64) -> anyhow::Result<(Schedule, bool)> {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let cur: Option<i64> = conn
        .query_row(
            "SELECT completed FROM schedules WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .optional()?;
    let cur = cur.ok_or_else(|| anyhow::anyhow!("日程不存在"))?;
    let new_val = if cur == 0 { 1 } else { 0 };
    let became_completed = new_val == 1;
    let completed_at = if became_completed { Some(now.clone()) } else { None };
    conn.execute(
        "UPDATE schedules SET completed = ?1, completed_at = ?2, updated_at = ?3 WHERE id = ?4",
        params![new_val, completed_at, now, id],
    )?;
    // 记录完成日志
    if became_completed {
        conn.execute(
            "INSERT INTO completion_log (schedule_id, completed_at) VALUES (?1, ?2)",
            params![id, now],
        )?;
    }
    let s = get_one(conn, id)?;
    Ok((s, became_completed))
}

/// 更新日程（编辑）
#[derive(Debug, Deserialize)]
pub struct UpdateSchedule {
    pub title: Option<String>,
    pub date: Option<String>,
    pub time_of_day: Option<Option<String>>,
    pub note: Option<Option<String>>,
    pub priority: Option<i64>,
    pub ddl_at: Option<Option<String>>,
    pub attachment: Option<Option<String>>,
}

pub fn update_schedule(conn: &Connection, id: i64, u: UpdateSchedule) -> anyhow::Result<Schedule> {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    // 逐字段判断更新，避免动态拼 SQL 的复杂性
    update_schedule_safe(conn, id, u, &now)
}

fn update_schedule_safe(
    conn: &Connection,
    id: i64,
    u: UpdateSchedule,
    now: &str,
) -> anyhow::Result<Schedule> {
    if let Some(title) = u.title {
        conn.execute(
            "UPDATE schedules SET title=?1, updated_at=?2 WHERE id=?3",
            params![title, now, id],
        )?;
    }
    if let Some(date) = u.date {
        conn.execute(
            "UPDATE schedules SET date=?1, updated_at=?2 WHERE id=?3",
            params![date, now, id],
        )?;
    }
    if let Some(time) = u.time_of_day {
        conn.execute(
            "UPDATE schedules SET time_of_day=?1, updated_at=?2 WHERE id=?3",
            params![time, now, id],
        )?;
    }
    if let Some(note) = u.note {
        conn.execute(
            "UPDATE schedules SET note=?1, updated_at=?2 WHERE id=?3",
            params![note, now, id],
        )?;
    }
    if let Some(priority) = u.priority {
        conn.execute(
            "UPDATE schedules SET priority=?1, updated_at=?2 WHERE id=?3",
            params![priority, now, id],
        )?;
    }
    if let Some(ddl) = u.ddl_at {
        let has_ddl = if ddl.is_some() { 1 } else { 0 };
        conn.execute(
            "UPDATE schedules SET ddl_at=?1, has_ddl=?2, updated_at=?3 WHERE id=?4",
            params![ddl, has_ddl, now, id],
        )?;
    }
    if let Some(att) = u.attachment {
        conn.execute(
            "UPDATE schedules SET attachment=?1, updated_at=?2 WHERE id=?3",
            params![att, now, id],
        )?;
    }
    get_one(conn, id)
}

/// 统计某 group 的日程数（用于判断是否多天，删除时提示）
pub fn count_by_group(conn: &Connection, group_id: &str) -> anyhow::Result<i64> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM schedules WHERE group_id=?1",
        params![group_id],
        |r| r.get(0),
    )?;
    Ok(n)
}

/// 删除单条日程
pub fn delete_schedule(conn: &Connection, id: i64) -> anyhow::Result<()> {
    conn.execute("DELETE FROM schedules WHERE id=?1", params![id])?;
    Ok(())
}

/// 按 group 删除（删除整组范围日程）
pub fn delete_group(conn: &Connection, group_id: &str) -> anyhow::Result<()> {
    conn.execute("DELETE FROM schedules WHERE group_id=?1", params![group_id])?;
    Ok(())
}

fn get_one(conn: &Connection, id: i64) -> anyhow::Result<Schedule> {
    let s = conn.query_row(
        "SELECT id, group_id, title, date, time_of_day, note, priority, has_ddl, ddl_at,
                completed, completed_at, attachment, created_at, updated_at
         FROM schedules WHERE id=?1",
        params![id],
        row_to_schedule,
    )?;
    Ok(s)
}

fn row_to_schedule(row: &rusqlite::Row<'_>) -> rusqlite::Result<Schedule> {
    Ok(Schedule {
        id: row.get(0)?,
        group_id: row.get(1)?,
        title: row.get(2)?,
        date: row.get(3)?,
        time_of_day: row.get(4)?,
        note: row.get(5)?,
        priority: row.get(6)?,
        has_ddl: row.get::<_, i64>(7)? != 0,
        ddl_at: row.get(8)?,
        completed: row.get::<_, i64>(9)? != 0,
        completed_at: row.get(10)?,
        attachment: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

// ============ 鼓励语 ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encouragement {
    pub id: i64,
    pub text: String,
    pub category: Option<String>,
    pub enabled: bool,
}

/// 随机抽取一条鼓励语（仅启用的）
pub fn random_encouragement(conn: &Connection) -> anyhow::Result<Option<Encouragement>> {
    let row = conn
        .query_row(
            "SELECT id, text, category, enabled FROM encouragements
             WHERE enabled=1 ORDER BY RANDOM() LIMIT 1",
            [],
            |r| {
                Ok(Encouragement {
                    id: r.get(0)?,
                    text: r.get(1)?,
                    category: r.get(2)?,
                    enabled: r.get::<_, i64>(3)? != 0,
                })
            },
        )
        .optional()?;
    Ok(row)
}

pub fn list_encouragements(conn: &Connection) -> anyhow::Result<Vec<Encouragement>> {
    let mut stmt = conn.prepare(
        "SELECT id, text, category, enabled FROM encouragements ORDER BY id ASC",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(Encouragement {
                id: r.get(0)?,
                text: r.get(1)?,
                category: r.get(2)?,
                enabled: r.get::<_, i64>(3)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

// ============ 导入 / 导出 ============

/// 导出全部日程为 JSON 字符串（含所有字段，保留 group_id 关联）
pub fn export_all(conn: &Connection) -> anyhow::Result<String> {
    let list = list_schedules_in_range(conn, "0000-01-01", "9999-12-31")?;
    // 导出格式：带版本号的包装，便于未来兼容
    let payload = serde_json::json!({
        "app": "desktop-schedule",
        "version": 1,
        "exported_at": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "schedules": list,
    });
    Ok(serde_json::to_string_pretty(&payload)?)
}

/// 导入的单条日程（id 由导入时重新生成，其余字段保留）
#[derive(Debug, Deserialize)]
pub struct ImportItem {
    pub group_id: Option<String>,
    pub title: String,
    pub date: String,
    pub time_of_day: Option<String>,
    pub note: Option<String>,
    #[serde(default)]
    pub priority: i64,
    #[serde(default)]
    pub has_ddl: bool,
    pub ddl_at: Option<String>,
    #[serde(default)]
    pub completed: bool,
    pub completed_at: Option<String>,
    pub attachment: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// 导入日程（批量插入）。group_id 保留以维持跨天关联。
/// 返回插入条数。
pub fn import_all(conn: &Connection, json_text: &str) -> anyhow::Result<usize> {
    // 兼容两种格式：包装对象 {schedules:[...]} 或裸数组 [...]
    let items: Vec<ImportItem> = if json_text.trim_start().starts_with('[') {
        serde_json::from_str(json_text)?
    } else {
        let v: serde_json::Value = serde_json::from_str(json_text)?;
        if let Some(arr) = v.get("schedules") {
            serde_json::from_value(arr.clone())?
        } else {
            return Err(anyhow::anyhow!("无法识别的导入格式"));
        }
    };

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let tx = conn.unchecked_transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO schedules
                (group_id, title, date, time_of_day, note, priority, has_ddl, ddl_at,
                 completed, completed_at, attachment, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        )?;
        for it in &items {
            stmt.execute(params![
                it.group_id,
                it.title,
                it.date,
                it.time_of_day,
                it.note,
                it.priority,
                it.has_ddl as i64,
                it.ddl_at,
                it.completed as i64,
                it.completed_at,
                it.attachment,
                it.created_at.as_ref().unwrap_or(&now),
                it.updated_at.as_ref().unwrap_or(&now),
            ])?;
        }
    }
    tx.commit()?;
    Ok(items.len())
}

