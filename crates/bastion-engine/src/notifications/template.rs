use sqlx::Row;
use sqlx::SqlitePool;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

pub(super) struct TemplateContext {
    title: String,
    job_id: String,
    job_name: String,
    run_id: String,
    status: String,
    status_text: String,
    started_at: String,
    ended_at: String,
    target_type: String,
    target_location: String,
    target: String,
    error: String,
    target_line_wecom: String,
    error_line_wecom: String,
    target_line_email: String,
    error_line_email: String,
}

pub(super) async fn build_context(
    db: &SqlitePool,
    run_id: &str,
) -> Result<TemplateContext, anyhow::Error> {
    let row = sqlx::query(
        "SELECT job_id, status, started_at, ended_at, error, summary_json FROM runs WHERE id = ? LIMIT 1",
    )
    .bind(run_id)
    .fetch_optional(db)
    .await?;

    let Some(row) = row else {
        return Ok(TemplateContext {
            title: "Bastion backup completed".to_string(),
            job_id: "-".to_string(),
            job_name: "-".to_string(),
            run_id: run_id.to_string(),
            status: "unknown".to_string(),
            status_text: "Unknown".to_string(),
            started_at: "-".to_string(),
            ended_at: "-".to_string(),
            target_type: "-".to_string(),
            target_location: "-".to_string(),
            target: "-".to_string(),
            error: String::new(),
            target_line_wecom: String::new(),
            error_line_wecom: String::new(),
            target_line_email: String::new(),
            error_line_email: String::new(),
        });
    };

    let job_id = row.get::<String, _>("job_id");
    let status = row.get::<String, _>("status");
    let started_at = row.get::<i64, _>("started_at");
    let ended_at = row.get::<Option<i64>, _>("ended_at");
    let error = row.get::<Option<String>, _>("error");
    let summary_json = row.get::<Option<String>, _>("summary_json");

    let job_name = sqlx::query_scalar::<_, String>("SELECT name FROM jobs WHERE id = ? LIMIT 1")
        .bind(&job_id)
        .fetch_optional(db)
        .await?
        .unwrap_or_else(|| job_id.clone());

    let (title, status_text) = match status.as_str() {
        "success" => (
            "Bastion backup succeeded".to_string(),
            "Succeeded".to_string(),
        ),
        "failed" => ("Bastion backup failed".to_string(), "Failed".to_string()),
        "rejected" => (
            "Bastion backup rejected".to_string(),
            "Rejected".to_string(),
        ),
        other => (
            format!("Bastion backup completed ({other})"),
            other.to_string(),
        ),
    };

    let started_at_str = format_ts(started_at);
    let ended_at_str = ended_at.map(format_ts).unwrap_or_else(|| "-".to_string());

    let mut target_type = "-".to_string();
    let mut target_location = "-".to_string();
    if let Some(summary) = summary_json
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(&summary)
        && let Some(target) = v.get("target")
    {
        target_type = target
            .get("type")
            .and_then(|x| x.as_str())
            .unwrap_or("-")
            .to_string();
        target_location = target
            .get("run_url")
            .or_else(|| target.get("run_dir"))
            .and_then(|x| x.as_str())
            .unwrap_or("-")
            .to_string();
    }

    let target = if target_type != "-" && target_location != "-" {
        format!("{target_type} {target_location}")
    } else if target_type != "-" {
        target_type.clone()
    } else if target_location != "-" {
        target_location.clone()
    } else {
        "-".to_string()
    };

    let error = error.unwrap_or_default();
    let error = error.trim().to_string();

    let target_line_wecom = if target != "-" {
        format!("> Target: {target}\n")
    } else {
        String::new()
    };
    let error_line_wecom = if !error.is_empty() {
        format!("> Error: {error}\n")
    } else {
        String::new()
    };

    let target_line_email = if target != "-" {
        format!("Target: {target}\n")
    } else {
        String::new()
    };
    let error_line_email = if !error.is_empty() {
        format!("Error: {error}\n")
    } else {
        String::new()
    };

    Ok(TemplateContext {
        title,
        job_id,
        job_name,
        run_id: run_id.to_string(),
        status,
        status_text,
        started_at: started_at_str,
        ended_at: ended_at_str,
        target_type,
        target_location,
        target,
        error,
        target_line_wecom,
        error_line_wecom,
        target_line_email,
        error_line_email,
    })
}

pub(super) fn render_template(template: &str, ctx: &TemplateContext) -> String {
    let pairs = [
        ("{{title}}", ctx.title.as_str()),
        ("{{job_id}}", ctx.job_id.as_str()),
        ("{{job_name}}", ctx.job_name.as_str()),
        ("{{run_id}}", ctx.run_id.as_str()),
        ("{{status}}", ctx.status.as_str()),
        ("{{status_text}}", ctx.status_text.as_str()),
        ("{{started_at}}", ctx.started_at.as_str()),
        ("{{ended_at}}", ctx.ended_at.as_str()),
        ("{{target_type}}", ctx.target_type.as_str()),
        ("{{target_location}}", ctx.target_location.as_str()),
        ("{{target}}", ctx.target.as_str()),
        ("{{error}}", ctx.error.as_str()),
        ("{{target_line_wecom}}", ctx.target_line_wecom.as_str()),
        ("{{error_line_wecom}}", ctx.error_line_wecom.as_str()),
        ("{{target_line_email}}", ctx.target_line_email.as_str()),
        ("{{error_line_email}}", ctx.error_line_email.as_str()),
    ];

    let mut out = template.to_string();
    for (k, v) in pairs {
        out = out.replace(k, v);
    }
    out
}

fn format_ts(ts: i64) -> String {
    OffsetDateTime::from_unix_timestamp(ts)
        .ok()
        .and_then(|t| t.format(&Rfc3339).ok())
        .unwrap_or_else(|| ts.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    use bastion_storage::jobs_repo::OverlapPolicy;
    use bastion_storage::runs_repo::RunStatus;

    #[test]
    fn format_ts_renders_rfc3339_and_falls_back_on_invalid_values() {
        assert_eq!(format_ts(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_ts(i64::MAX), i64::MAX.to_string());
    }

    #[test]
    fn render_template_replaces_known_placeholders() {
        let ctx = TemplateContext {
            title: "t".to_string(),
            job_id: "j".to_string(),
            job_name: "jn".to_string(),
            run_id: "r".to_string(),
            status: "s".to_string(),
            status_text: "st".to_string(),
            started_at: "sa".to_string(),
            ended_at: "ea".to_string(),
            target_type: "tt".to_string(),
            target_location: "tl".to_string(),
            target: "tgt".to_string(),
            error: "err".to_string(),
            target_line_wecom: "> Target: tgt\n".to_string(),
            error_line_wecom: "> Error: err\n".to_string(),
            target_line_email: "Target: tgt\n".to_string(),
            error_line_email: "Error: err\n".to_string(),
        };

        let out = render_template("{{title}} {{job_id}} {{run_id}} {{unknown}}", &ctx);
        assert_eq!(out, "t j r {{unknown}}");
    }

    async fn init_test_db() -> Result<(tempfile::TempDir, SqlitePool), anyhow::Error> {
        let dir = tempfile::TempDir::new()?;
        let db = bastion_storage::db::init(dir.path()).await?;
        Ok((dir, db))
    }

    #[tokio::test]
    async fn build_context_returns_defaults_when_run_missing() -> Result<(), anyhow::Error> {
        let (_dir, db) = init_test_db().await?;

        let ctx = build_context(&db, "run_missing").await?;
        assert_eq!(ctx.run_id, "run_missing");
        assert_eq!(ctx.status, "unknown");
        assert_eq!(ctx.job_id, "-");
        assert_eq!(ctx.job_name, "-");
        assert_eq!(ctx.started_at, "-");
        assert_eq!(ctx.ended_at, "-");
        assert_eq!(ctx.target, "-");
        assert_eq!(ctx.error, "");
        assert_eq!(ctx.target_line_email, "");
        assert_eq!(ctx.error_line_email, "");
        Ok(())
    }

    #[tokio::test]
    async fn build_context_builds_target_and_error_lines() -> Result<(), anyhow::Error> {
        let (_dir, db) = init_test_db().await?;

        let job = bastion_storage::jobs_repo::create_job(
            &db,
            "myjob",
            None,
            None,
            None,
            OverlapPolicy::Reject,
            serde_json::json!({}),
        )
        .await?;

        let summary = serde_json::json!({
            "target": {
                "type": "webdav",
                "run_url": "https://example.invalid/runs/123"
            }
        });
        let run = bastion_storage::runs_repo::create_run(
            &db,
            &job.id,
            RunStatus::Success,
            0,
            Some(1),
            Some(summary),
            Some("  boom \n"),
        )
        .await?;

        let ctx = build_context(&db, &run.id).await?;
        assert_eq!(ctx.title, "Bastion backup succeeded");
        assert_eq!(ctx.job_id, job.id);
        assert_eq!(ctx.job_name, "myjob");
        assert_eq!(ctx.status, "success");
        assert_eq!(ctx.status_text, "Succeeded");
        assert_eq!(ctx.started_at, "1970-01-01T00:00:00Z");
        assert_eq!(ctx.ended_at, "1970-01-01T00:00:01Z");

        assert_eq!(ctx.target_type, "webdav");
        assert_eq!(ctx.target_location, "https://example.invalid/runs/123");
        assert_eq!(ctx.target, "webdav https://example.invalid/runs/123");

        assert_eq!(ctx.error, "boom");
        assert_eq!(
            ctx.target_line_wecom,
            "> Target: webdav https://example.invalid/runs/123\n"
        );
        assert_eq!(ctx.error_line_wecom, "> Error: boom\n");
        assert_eq!(
            ctx.target_line_email,
            "Target: webdav https://example.invalid/runs/123\n"
        );
        assert_eq!(ctx.error_line_email, "Error: boom\n");
        Ok(())
    }
}
