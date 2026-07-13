import type { DetailListItem } from "@/lib/api/types";

export interface DashboardFilters {
  status: number;
  keyword: string;
  sortBy: "name" | "time";
  sortOrder: "asc" | "desc";
}

export function filterRecords(
  records: DetailListItem[],
  filters: DashboardFilters,
) {
  const keyword = filters.keyword.trim().toLocaleLowerCase("zh-CN");
  const result = records.filter((record) => {
    if (
      record.is_delete ||
      (filters.status !== -1 && record.user_status !== filters.status)
    )
      return false;
    if (!keyword) return true;
    return [record.title, record.bangumi_id, record.imdb_id, record.other_id]
      .filter((value) => value !== null && value !== undefined)
      .some((value) =>
        String(value).toLocaleLowerCase("zh-CN").includes(keyword),
      );
  });
  return result.sort((left, right) => {
    const direction = filters.sortOrder === "asc" ? 1 : -1;
    if (filters.sortBy === "name")
      return (
        direction *
        (left.title || "").localeCompare(right.title || "", "zh-Hans-CN")
      );
    return (
      direction *
      (new Date(left.updated_at).getTime() -
        new Date(right.updated_at).getTime())
    );
  });
}

export function recordHref(record: DetailListItem) {
  if ((record.source === "bangumi" || record.bangumi_id) && record.bangumi_id)
    return `/detail/${record.bangumi_id}`;
  if ((record.source === "imdb" || record.imdb_id) && record.imdb_id)
    return `/detail/imdb/${encodeURIComponent(record.imdb_id)}`;
  if (record.other_id) return `/detail/custom/${record.other_id}`;
  return null;
}
