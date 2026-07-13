import { describe, expect, it } from "vitest";
import type { DetailListItem } from "@/lib/api/types";
import { filterRecords, recordHref } from "./model";

function record(patch: Partial<DetailListItem>): DetailListItem {
  return {
    id: 1,
    source: "bangumi",
    external_id: null,
    local_external_media_id: null,
    local_bangumi_id: null,
    other_id: null,
    bangumi_id: "100",
    imdb_id: null,
    title: "作品 A",
    type: 1,
    author: null,
    episodes: 12,
    cover_url: null,
    recorder: "3",
    user_status: 1,
    is_delete: false,
    updated_at: "2026-07-01 10:00:00",
    created_at: "2026-06-01 10:00:00",
    ...patch,
  };
}

describe("dashboard model", () => {
  const items = [
    record({ id: 1, title: "作品 B", user_status: 2 }),
    record({
      id: 2,
      title: "作品 A",
      bangumi_id: "200",
      updated_at: "2026-07-02 10:00:00",
    }),
    record({ id: 3, is_delete: true }),
  ];

  it("filters soft-deleted rows and status", () => {
    expect(
      filterRecords(items, {
        status: -1,
        keyword: "",
        sortBy: "time",
        sortOrder: "desc",
      }).map((item) => item.id),
    ).toEqual([2, 1]);
    expect(
      filterRecords(items, {
        status: 2,
        keyword: "",
        sortBy: "time",
        sortOrder: "desc",
      }).map((item) => item.id),
    ).toEqual([1]);
  });

  it("searches IDs and sorts Chinese titles", () => {
    expect(
      filterRecords(items, {
        status: -1,
        keyword: "200",
        sortBy: "name",
        sortOrder: "asc",
      }).map((item) => item.id),
    ).toEqual([2]);
    expect(
      filterRecords(items, {
        status: -1,
        keyword: "",
        sortBy: "name",
        sortOrder: "asc",
      }).map((item) => item.title),
    ).toEqual(["作品 A", "作品 B"]);
  });

  it("keeps all three legacy detail URL shapes", () => {
    expect(recordHref(record({ bangumi_id: "42" }))).toBe("/detail/42");
    expect(
      recordHref(
        record({ source: "imdb", bangumi_id: null, imdb_id: "tt 42" }),
      ),
    ).toBe("/detail/imdb/tt%2042");
    expect(
      recordHref(record({ source: "custom", bangumi_id: null, other_id: 7 })),
    ).toBe("/detail/custom/7");
  });
});
