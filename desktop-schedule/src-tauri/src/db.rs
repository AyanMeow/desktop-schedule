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

/// 打开/创建数据库并执行建表迁移
pub fn open(path: &std::path::Path) -> anyhow::Result<Db> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    migrate(&conn)?;
    seed_encouragements(&conn)?;
    Ok(Db::new(conn))
}

fn migrate(conn: &Connection) -> anyhow::Result<()> {
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
    // 兼容旧库：若 attachment 列不存在则补上
    let has_attachment: bool = conn
        .prepare("PRAGMA table_info(schedules)")?
        .query_map([], |r| r.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .any(|name| name == "attachment");
    if !has_attachment {
        conn.execute("ALTER TABLE schedules ADD COLUMN attachment TEXT", [])?;
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
        // 励志
        ("完成一件事，就是给自己最好的礼物。", "励志"),
        ("又搞定一项！你比昨天更厉害了。", "励志"),
        ("千里之行，始于足下；这一步走得很稳。", "励志"),
        ("积少成多，你正在变得更强。", "励志"),
        ("今天的努力，是明天的底气。", "励志"),
        ("每一个完成，都是对未来的投资。", "励志"),
        ("你做到了，为你骄傲！", "励志"),
        ("坚持下去，量变终会引发质变。", "励志"),
        // 幽默
        ("叮！经验值 +1，离升级又近一步。", "幽默"),
        ("这任务看到你上线，自己投降了。", "幽默"),
        ("恭喜达成成就：把待办变成已办。", "幽默"),
        ("今日战斗力评测：MAX。", "幽默"),
        ("这效率，老板看了都想加薪。", "幽默"),
        ("打卡成功，奖励自己一杯奶茶不过分吧？", "幽默"),
        // 温柔
        ("慢慢来，比较快。你做得很好。", "温柔"),
        ("记得喝水，你辛苦啦。", "温柔"),
        ("今天也要好好吃饭，好好休息。", "温柔"),
        ("别太累，完成一件事就够了。", "温柔"),
        ("你已经很棒了，不必太苛求自己。", "温柔"),
        ("深呼吸，一切都在慢慢变好。", "温柔"),
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
