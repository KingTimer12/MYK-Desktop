/**
 * Sistema de Extensões WebAssembly para MangaYouKnow
 *
 * Este módulo fornece funções para gerenciar e executar extensões WASM.
 */

import { invoke } from "@tauri-apps/api/core";

// ============================================================================
// Tipos e Interfaces
// ============================================================================

export enum ContentType {
  Manga = "manga",
  Comic = "comic",
  Anime = "anime",
}

export interface Language {
  id: string;
  label: string;
}

export interface Chapter {
  number: string;
  title?: string;
  chapterId: string;
  source: string;
  language?: string;
  scan?: string;
  path?: string;
  thumbnail?: string;
}

export interface Favorite {
  id: number;
  userId?: number;
  name: string;
  folderName: string;
  link: string;
  cover: string;
  source: string;
  sourceId: string;
  type?: ContentType;
  extraName?: string;
  titleColor?: string;
  cardColor?: string;
  malId?: string;
  anilistId?: string;
  status?: string;
  grade?: number;
  author?: string;
  isUltraFavorite?: boolean | string;
  description?: string;
  marks?: any[];
  readeds?: any[];
}

export interface ExtensionExports {
  search: boolean;
  getChapters: boolean;
  getChapterImages: boolean;
  isMultiLanguage: boolean;
}

export interface ExtensionInfo {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  nsfw: boolean;
  language: string;
  extensionType: string;
  baseUrl: string;
  installed: boolean;
  loaded: boolean;
  exports: ExtensionExports;
}

// ============================================================================
// Funções de Gerenciamento de Extensões
// ============================================================================

/**
 * Lista todas as extensões instaladas
 */
export async function listInstalledExtensions(): Promise<ExtensionInfo[]> {
  return await invoke<ExtensionInfo[]>("list_installed_extensions");
}

/**
 * Lista extensões carregadas no runtime
 */
export async function listLoadedExtensions(): Promise<string[]> {
  return await invoke<string[]>("list_loaded_extensions");
}

/**
 * Carrega uma extensão no runtime
 */
export async function loadExtension(
  extensionId: string,
): Promise<ExtensionInfo> {
  return await invoke<ExtensionInfo>("load_extension", { extensionId });
}

/**
 * Descarrega uma extensão do runtime
 */
export async function unloadExtension(extensionId: string): Promise<void> {
  await invoke("unload_extension", { extensionId });
}

/**
 * Recarrega uma extensão (útil durante desenvolvimento)
 */
export async function reloadExtension(
  extensionId: string,
): Promise<ExtensionInfo> {
  return await invoke<ExtensionInfo>("reload_extension", { extensionId });
}

/**
 * Obtém informações detalhadas de uma extensão
 */
export async function getExtensionInfo(
  extensionId: string,
): Promise<ExtensionInfo> {
  return await invoke<ExtensionInfo>("get_extension_info", { extensionId });
}

/**
 * Instala uma extensão a partir de arquivos locais
 */
export async function installExtensionFromFile(
  wasmPath: string,
  manifestPath: string,
): Promise<ExtensionInfo> {
  return await invoke<ExtensionInfo>("install_extension_from_file", {
    wasmPath,
    manifestPath,
  });
}

/**
 * Desinstala uma extensão
 */
export async function uninstallExtension(extensionId: string): Promise<void> {
  await invoke("uninstall_extension", { extensionId });
}

/**
 * Obtém o diretório onde as extensões são armazenadas
 */
export async function getExtensionsDirectory(): Promise<string> {
  return await invoke<string>("get_extensions_directory");
}

// ============================================================================
// Funções de Execução de Extensões
// ============================================================================

/**
 * Busca mangás/animes usando uma extensão
 */
export async function extensionSearch(
  extensionId: string,
  query: string,
): Promise<Favorite[]> {
  return await invoke<Favorite[]>("extension_search", {
    extensionId,
    query,
  });
}

/**
 * Obtém capítulos de um mangá/anime
 */
export async function extensionGetChapters(
  extensionId: string,
  favoriteId: string,
  language?: string,
): Promise<Chapter[]> {
  return await invoke<Chapter[]>("extension_get_chapters", {
    extensionId,
    favoriteId,
    language,
  });
}

/**
 * Obtém imagens de um capítulo
 */
export async function extensionGetChapterImages(
  extensionId: string,
  chapterId: string,
): Promise<string[]> {
  return await invoke<string[]>("extension_get_chapter_images", {
    extensionId,
    chapterId,
  });
}

/**
 * Obtém idiomas disponíveis para um mangá/anime
 */
export async function extensionGetLanguages(
  extensionId: string,
  favoriteId: string,
): Promise<Language[]> {
  return await invoke<Language[]>("extension_get_languages", {
    extensionId,
    favoriteId,
  });
}

// ============================================================================
// Classe Helper para facilitar uso
// ============================================================================

export class Extension {
  constructor(
    public readonly id: string,
    public info: ExtensionInfo,
  ) {}

  /**
   * Verifica se a extensão está carregada
   */
  get isLoaded(): boolean {
    return this.info.loaded;
  }

  /**
   * Carrega a extensão se não estiver carregada
   */
  async ensureLoaded(): Promise<void> {
    if (!this.isLoaded) {
      this.info = await loadExtension(this.id);
    }
  }

  /**
   * Busca conteúdo
   */
  async search(query: string): Promise<Favorite[]> {
    await this.ensureLoaded();
    return await extensionSearch(this.id, query);
  }

  /**
   * Obtém capítulos
   */
  async getChapters(favoriteId: string, language?: string): Promise<Chapter[]> {
    await this.ensureLoaded();
    return await extensionGetChapters(this.id, favoriteId, language);
  }

  /**
   * Obtém imagens de um capítulo
   */
  async getChapterImages(chapterId: string): Promise<string[]> {
    await this.ensureLoaded();
    return await extensionGetChapterImages(this.id, chapterId);
  }

  /**
   * Obtém idiomas disponíveis
   */
  async getLanguages(favoriteId: string): Promise<Language[]> {
    await this.ensureLoaded();
    return await extensionGetLanguages(this.id, favoriteId);
  }

  /**
   * Descarrega a extensão
   */
  async unload(): Promise<void> {
    await unloadExtension(this.id);
    this.info.loaded = false;
  }

  /**
   * Recarrega a extensão
   */
  async reload(): Promise<void> {
    this.info = await reloadExtension(this.id);
  }
}

// ============================================================================
// Extension Manager - Gerenciador de extensões
// ============================================================================

export class ExtensionManager {
  private extensions: Map<string, Extension> = new Map();

  /**
   * Inicializa o manager carregando todas as extensões instaladas
   */
  async initialize(): Promise<void> {
    const installed = await listInstalledExtensions();
    for (const info of installed) {
      this.extensions.set(info.id, new Extension(info.id, info));
    }
  }

  /**
   * Obtém todas as extensões
   */
  getAll(): Extension[] {
    return Array.from(this.extensions.values());
  }

  /**
   * Obtém uma extensão por ID
   */
  get(id: string): Extension | undefined {
    return this.extensions.get(id);
  }

  /**
   * Obtém extensões carregadas
   */
  getLoaded(): Extension[] {
    return this.getAll().filter((ext) => ext.isLoaded);
  }

  /**
   * Obtém extensões de um tipo específico
   */
  getByType(type: string): Extension[] {
    return this.getAll().filter((ext) => ext.info.extensionType === type);
  }

  /**
   * Carrega uma extensão
   */
  async load(id: string): Promise<Extension> {
    let ext = this.extensions.get(id);
    if (!ext) {
      const info = await loadExtension(id);
      ext = new Extension(id, info);
      this.extensions.set(id, ext);
    } else {
      await ext.ensureLoaded();
    }
    return ext;
  }

  /**
   * Descarrega uma extensão
   */
  async unload(id: string): Promise<void> {
    const ext = this.extensions.get(id);
    if (ext) {
      await ext.unload();
    }
  }

  /**
   * Recarrega todas as extensões instaladas
   */
  async refresh(): Promise<void> {
    await this.initialize();
  }
}

// ============================================================================
// Utilitários
// ============================================================================

/**
 * Verifica se uma extensão suporta busca
 */
export function supportsSearch(extension: ExtensionInfo | Extension): boolean {
  const info = extension instanceof Extension ? extension.info : extension;
  return info.exports.search;
}

/**
 * Verifica se uma extensão suporta multi-idioma
 */
export function supportsMultiLanguage(
  extension: ExtensionInfo | Extension,
): boolean {
  const info = extension instanceof Extension ? extension.info : extension;
  return info.exports.isMultiLanguage;
}

/**
 * Verifica se uma extensão suporta capítulos
 */
export function supportsChapters(
  extension: ExtensionInfo | Extension,
): boolean {
  const info = extension instanceof Extension ? extension.info : extension;
  return info.exports.getChapters;
}

/**
 * Verifica se uma extensão suporta imagens
 */
export function supportsImages(extension: ExtensionInfo | Extension): boolean {
  const info = extension instanceof Extension ? extension.info : extension;
  return info.exports.getChapterImages;
}
