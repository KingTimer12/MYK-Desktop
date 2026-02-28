import { invoke } from "@tauri-apps/api/core";
import type { MangaDl } from "@/interfaces";
import type { Favorite, Chapter, Language } from "@/types";

/**
 * MangaFire downloader implementation using Rust backend
 * This implementation uses the Rust ApiClient with scraper for HTML parsing
 */
export class MangaFireRustDl implements MangaDl {
  baseUrl = "https://mangafire.to";
  isMultiLanguage = true;

  async search(query: string): Promise<Favorite[]> {
    return await invoke<Favorite[]>("mangafire_search", { query });
  }

  async getFavoriteLanguages(favoriteId: string): Promise<Language[]> {
    return await invoke<Language[]>("mangafire_get_favorite_languages", {
      favoriteId: favoriteId,
    });
  }

  async getChapters(
    favoriteId: string,
    language?: string
  ): Promise<Chapter[]> {
    return await invoke<Chapter[]>("mangafire_get_chapters", {
      favoriteId: favoriteId,
      language: language ?? null,
    });
  }

  async getChapterImages(chapterId: string): Promise<string[]> {
    return await invoke<string[]>("mangafire_get_chapter_images", {
      chapterId: chapterId,
    });
  }

  async getMangaByUrl(url: string): Promise<Favorite> {
    return await invoke<Favorite>("mangafire_get_manga_by_url", { url });
  }

  async getMangaById(id: string): Promise<Favorite> {
    return await invoke<Favorite>("mangafire_get_manga_by_id", { id });
  }
}
