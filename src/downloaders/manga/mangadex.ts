import { invoke } from "@tauri-apps/api/core";
import type { MangaDl } from "@/interfaces";
import type { Favorite, Chapter, Language } from "@/types";

export class MangaDexDl implements MangaDl {
  baseUrl = "https://mangadex.org";
  isMultiLanguage = true;

  async search(query: string): Promise<Favorite[]> {
    return await invoke<Favorite[]>("mangadex_search", { query });
  }

  async getFavoriteLanguages(favoriteId: string): Promise<Language[]> {
    return await invoke<Language[]>("mangadex_get_favorite_languages", {
      favoriteId: favoriteId,
    });
  }

  async getChapters(favoriteId: string, language?: string): Promise<Chapter[]> {
    return await invoke<Chapter[]>("mangadex_get_chapters", {
      favoriteId: favoriteId,
      language: language ?? null,
    });
  }

  async getChapterImages(chapterId: string): Promise<string[]> {
    return await invoke<string[]>("mangadex_get_chapter_images", {
      chapterId: chapterId,
    });
  }

  async getMangaByUrl(url: string): Promise<Favorite> {
    return await invoke<Favorite>("mangadex_get_manga_by_url", { url });
  }

  async getMangaById(id: string): Promise<Favorite> {
    return await invoke<Favorite>("mangadex_get_manga_by_id", { id });
  }
}
