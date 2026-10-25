import { save } from "@tauri-apps/plugin-dialog";
import { api, errMessage } from "./api";
import { toast } from "./stores";
import { formatRange } from "./format";

const EXT: Record<string, string> = { csv: "csv", json: "json", md: "md" };
const FILTER_NAME: Record<string, string> = {
  csv: "CSV",
  json: "JSON",
  md: "Markdown",
};

export async function exportRange(
  format: "csv" | "json" | "md",
  startDay: string,
  endDay: string,
  projectId: number | null = null,
  unassigned = false,
  projectSlug = "all-projects",
): Promise<string | null> {
  try {
    const defaultName = `Framewise ${startDay} to ${endDay} - ${projectSlug}.${EXT[format]}`;
    const path = await save({
      title: "Export Framewise report",
      defaultPath: defaultName,
      filters: [{ name: FILTER_NAME[format], extensions: [EXT[format]] }],
    });
    if (!path) return null;
    const written = await api.exportData(
      format,
      startDay,
      endDay,
      projectId,
      unassigned,
      path,
    );
    toast(`Exported ${formatRange(startDay, endDay)}.`, "success");
    return written;
  } catch (error) {
    toast(errMessage(error), "error");
    return null;
  }
}

export function summarizeExportRange(startDay: string, endDay: string): string {
  return formatRange(startDay, endDay);
}
