use std::path::Path;

use anyhow::{anyhow, Result};
use serde::Serialize;

use crate::models::{Report, Session};
use crate::timeutil::{duration_label, local_datetime_string, local_time};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonExport<'a> {
    app: &'static str,
    version: &'static str,
    generated_at: String,
    report: &'a Report,
    sessions: &'a [Session],
}

pub fn export_report(
    format: &str,
    report: &Report,
    sessions: &[Session],
    path: &Path,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = match format {
        "csv" => to_csv(sessions),
        "json" => {
            let payload = JsonExport {
                app: "Framewise",
                version: crate::state::VERSION,
                generated_at: local_datetime_string(chrono::Local::now().timestamp()),
                report,
                sessions,
            };
            serde_json::to_string_pretty(&payload)?
        }
        "md" | "markdown" => to_markdown(report),
        other => return Err(anyhow!("unknown export format: {other}")),
    };
    std::fs::write(path, content)?;
    Ok(())
}

pub fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn to_csv(sessions: &[Session]) -> String {
    let mut out =
        String::from("date,start,end,duration_seconds,duration,app,project,source,adobe\n");
    for s in sessions {
        let duration = (s.ended_at - s.started_at).max(0);
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            s.day,
            local_time(s.started_at),
            local_time(s.ended_at),
            duration,
            hms(duration),
            escape_csv(&s.app),
            escape_csv(s.project_name.as_deref().unwrap_or("Unassigned")),
            escape_csv(&s.source),
            if s.adobe { "yes" } else { "no" }
        ));
    }
    out
}

fn hms(seconds: i64) -> String {
    let s = seconds.max(0);
    format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}

fn to_markdown(report: &Report) -> String {
    let mut out = String::new();
    out.push_str("# Framewise report\n\n");
    out.push_str(&format!(
        "**{} to {}**  \nGenerated locally by Framewise. No data left this machine.\n\n",
        report.start_day, report.end_day
    ));
    out.push_str(&format!(
        "## Summary\n\n- Total: **{}**\n",
        duration_label(report.total_seconds)
    ));
    if let Some(best) = &report.best_day {
        out.push_str(&format!(
            "- Best day: **{}** ({})\n",
            best.day,
            duration_label(best.seconds)
        ));
    }
    out.push_str(&format!("- Sessions: {}\n", report.session_count));
    if let (Some(first), Some(last)) = (report.first_activity, report.last_activity) {
        out.push_str(&format!(
            "- First activity: {}\n- Last activity: {}\n",
            local_datetime_string(first),
            local_datetime_string(last)
        ));
    }
    out.push('\n');

    if !report.goals.is_empty() {
        out.push_str("## Goals\n\n| Goal | Period | Progress | Target | Reached |\n|---|---|---:|---:|---|\n");
        for g in &report.goals {
            let scope = g.goal.project_name.as_deref().unwrap_or("All projects");
            out.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                scope,
                g.goal.period,
                duration_label(g.seconds),
                duration_label(g.target_seconds),
                if g.reached { "yes" } else { "no" }
            ));
        }
        out.push('\n');
    }

    if !report.per_project.is_empty() {
        out.push_str("## Projects\n\n| Project | Time | Share |\n|---|---:|---:|\n");
        for p in &report.per_project {
            out.push_str(&format!(
                "| {} | {} | {:.1}% |\n",
                p.name,
                duration_label(p.seconds),
                percent_of(p.seconds, report.total_seconds)
            ));
        }
        out.push('\n');
    }

    if !report.per_app.is_empty() {
        out.push_str("## Apps\n\n| App | Time | Share |\n|---|---:|---:|\n");
        for a in &report.per_app {
            out.push_str(&format!(
                "| {} | {} | {:.1}% |\n",
                a.app,
                duration_label(a.seconds),
                percent_of(a.seconds, report.total_seconds)
            ));
        }
        out.push('\n');
    }

    if !report.per_day.is_empty() {
        out.push_str("## Days\n\n| Day | Time |\n|---|---:|\n");
        for d in &report.per_day {
            out.push_str(&format!("| {} | {} |\n", d.day, duration_label(d.seconds)));
        }
        out.push('\n');
    }

    out.push_str("---\n\nMade by Sneg HQ.\n");
    out
}

fn percent_of(part: i64, total: i64) -> f64 {
    if total <= 0 {
        0.0
    } else {
        part as f64 / total as f64 * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_escaping_handles_commas_quotes_and_newlines() {
        assert_eq!(escape_csv("Premiere"), "Premiere");
        assert_eq!(escape_csv("A, B"), "\"A, B\"");
        assert_eq!(escape_csv("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(escape_csv("one\ntwo"), "\"one\ntwo\"");
    }

    #[test]
    fn duration_formats_to_hms() {
        assert_eq!(hms(0), "00:00:00");
        assert_eq!(hms(3661), "01:01:01");
        assert_eq!(hms(-5), "00:00:00");
    }

    #[test]
    fn percentages_are_safe() {
        assert_eq!(percent_of(0, 0), 0.0);
        assert!((percent_of(1800, 3600) - 50.0).abs() < 0.001);
    }
}
