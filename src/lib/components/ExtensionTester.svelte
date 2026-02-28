<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import {
    ExtensionManager,
    getExtensionsDirectory,
    installExtensionFromFile,
    uninstallExtension as uninstallExt,
    type ExtensionInfo,
    type Favorite,
    type Chapter,
    type Language,
  } from '$lib/extensions';

  const manager = new ExtensionManager();

  let extensions: ExtensionInfo[] = [];
  let selectedExtension: string = '';
  let loading = false;
  let error = '';
  let extensionsDir = '';

  // Estado de busca
  let searchQuery = '';
  let searchResults: Favorite[] = [];

  // Estado de capítulos
  let selectedFavorite: Favorite | null = null;
  let chapters: Chapter[] = [];
  let languages: Language[] = [];
  let selectedLanguage: string | undefined = undefined;

  // Estado de imagens
  let selectedChapter: Chapter | null = null;
  let images: string[] = [];

  // Logs
  let logs: string[] = [];

  function addLog(message: string) {
    const timestamp = new Date().toLocaleTimeString();
    logs = [`[${timestamp}] ${message}`, ...logs];
    console.log(message);
  }

  onMount(async () => {
    try {
      addLog('Inicializando Extension Manager...');
      await manager.initialize();
      extensions = manager.getAll().map((ext) => ext.info);
      addLog(`${extensions.length} extensões encontradas`);

      // Obter diretório de extensões
      extensionsDir = await getExtensionsDirectory();
      addLog(`Diretório de extensões: ${extensionsDir}`);

      if (extensions.length > 0) {
        selectedExtension = extensions[0].id;
      }
    } catch (e) {
      error = `Erro ao inicializar: ${e}`;
      addLog(`ERRO: ${error}`);
    }
  });

  async function handleLoadExtension() {
    if (!selectedExtension) return;

    loading = true;
    error = '';

    try {
      addLog(`Carregando extensão: ${selectedExtension}`);
      const ext = await manager.load(selectedExtension);
      addLog(`✓ Extensão ${ext.info.name} carregada com sucesso`);

      // Atualizar lista
      extensions = manager.getAll().map((ext) => ext.info);
    } catch (e) {
      error = `Erro ao carregar extensão: ${e}`;
      addLog(`ERRO: ${error}`);
    } finally {
      loading = false;
    }
  }

  async function handleUnloadExtension() {
    if (!selectedExtension) return;

    loading = true;
    error = '';

    try {
      addLog(`Descarregando extensão: ${selectedExtension}`);
      await manager.unload(selectedExtension);
      addLog(`✓ Extensão descarregada`);

      // Atualizar lista
      extensions = manager.getAll().map((ext) => ext.info);
    } catch (e) {
      error = `Erro ao descarregar: ${e}`;
      addLog(`ERRO: ${error}`);
    } finally {
      loading = false;
    }
  }

  async function handleSearch() {
    if (!selectedExtension || !searchQuery) return;

    loading = true;
    error = '';
    searchResults = [];

    try {
      addLog(`Buscando por: "${searchQuery}"`);
      const ext = manager.get(selectedExtension);
      if (!ext) throw new Error('Extensão não encontrada');

      searchResults = await ext.search(searchQuery);
      addLog(`✓ ${searchResults.length} resultados encontrados`);
    } catch (e) {
      error = `Erro na busca: ${e}`;
      addLog(`ERRO: ${error}`);
    } finally {
      loading = false;
    }
  }

  async function handleSelectFavorite(favorite: Favorite) {
    selectedFavorite = favorite;
    chapters = [];
    languages = [];
    selectedLanguage = undefined;

    loading = true;
    error = '';

    try {
      const ext = manager.get(selectedExtension);
      if (!ext) throw new Error('Extensão não encontrada');

      // Buscar idiomas se suportar multi-idioma
      if (ext.info.exports.isMultiLanguage) {
        addLog(`Buscando idiomas disponíveis...`);
        languages = await ext.getLanguages(favorite.sourceId);
        addLog(`✓ ${languages.length} idiomas disponíveis`);

        if (languages.length > 0) {
          selectedLanguage = languages[0].id;
        }
      }

      // Buscar capítulos
      addLog(`Buscando capítulos...`);
      chapters = await ext.getChapters(favorite.sourceId, selectedLanguage);
      addLog(`✓ ${chapters.length} capítulos encontrados`);
    } catch (e) {
      error = `Erro ao obter capítulos: ${e}`;
      addLog(`ERRO: ${error}`);
    } finally {
      loading = false;
    }
  }

  async function handleChangeLanguage() {
    if (!selectedFavorite || !selectedLanguage) return;

    loading = true;
    error = '';
    chapters = [];

    try {
      const ext = manager.get(selectedExtension);
      if (!ext) throw new Error('Extensão não encontrada');

      addLog(`Buscando capítulos em ${selectedLanguage}...`);
      chapters = await ext.getChapters(selectedFavorite.sourceId, selectedLanguage);
      addLog(`✓ ${chapters.length} capítulos encontrados`);
    } catch (e) {
      error = `Erro ao obter capítulos: ${e}`;
      addLog(`ERRO: ${error}`);
    } finally {
      loading = false;
    }
  }

  async function handleSelectChapter(chapter: Chapter) {
    selectedChapter = chapter;
    images = [];

    loading = true;
    error = '';

    try {
      const ext = manager.get(selectedExtension);
      if (!ext) throw new Error('Extensão não encontrada');

      addLog(`Buscando imagens do capítulo ${chapter.number}...`);
      images = await ext.getChapterImages(chapter.chapterId);
      addLog(`✓ ${images.length} imagens encontradas`);
    } catch (e) {
      error = `Erro ao obter imagens: ${e}`;
      addLog(`ERRO: ${error}`);
    } finally {
      loading = false;
    }
  }

  function clearLogs() {
    logs = [];
  }

  async function handleInstallExtension() {
    loading = true;
    error = '';

    try {
      addLog('Selecionando arquivo WASM...');
      const wasmFile = await open({
        title: 'Selecione o arquivo WASM da extensão',
        filters: [{
          name: 'WebAssembly',
          extensions: ['wasm']
        }],
        multiple: false,
      });

      if (!wasmFile) {
        addLog('Instalação cancelada');
        loading = false;
        return;
      }

      addLog('Selecionando arquivo manifest.json...');
      const manifestFile = await open({
        title: 'Selecione o arquivo manifest.json',
        filters: [{
          name: 'JSON',
          extensions: ['json']
        }],
        multiple: false,
      });

      if (!manifestFile) {
        addLog('Instalação cancelada');
        loading = false;
        return;
      }

      addLog(`Instalando extensão...`);
      const info = await installExtensionFromFile(wasmFile.path, manifestFile.path);
      addLog(`✓ Extensão ${info.name} instalada com sucesso!`);

      // Atualizar lista
      await manager.refresh();
      extensions = manager.getAll().map((ext) => ext.info);
      selectedExtension = info.id;
    } catch (e) {
      error = `Erro ao instalar extensão: ${e}`;
      addLog(`ERRO: ${error}`);
    } finally {
      loading = false;
    }
  }

  async function handleUninstallExtension() {
    if (!selectedExtension) return;

    if (!confirm(`Deseja realmente desinstalar a extensão "${selectedExtension}"?`)) {
      return;
    }

    loading = true;
    error = '';

    try {
      addLog(`Desinstalando extensão: ${selectedExtension}`);
      await uninstallExt(selectedExtension);
      addLog(`✓ Extensão desinstalada com sucesso`);

      // Atualizar lista
      await manager.refresh();
      extensions = manager.getAll().map((ext) => ext.info);

      if (extensions.length > 0) {
        selectedExtension = extensions[0].id;
      } else {
        selectedExtension = '';
      }
    } catch (e) {
      error = `Erro ao desinstalar: ${e}`;
      addLog(`ERRO: ${error}`);
    } finally {
      loading = false;
    }
  }
</script>

<div class="extension-tester">
  <h1>🧪 Testador de Extensões WASM</h1>

  {#if error}
    <div class="error">
      ⚠️ {error}
    </div>
  {/if}

  <!-- Seleção de Extensão -->
  <div class="section">
    <h2>1. Selecionar Extensão</h2>
    <div class="controls">
      <select bind:value={selectedExtension} disabled={loading}>
        {#each extensions as ext}
          <option value={ext.id}>
            {ext.name} v{ext.version}
            {ext.loaded ? '✓ Carregada' : '○ Não carregada'}
          </option>
        {/each}
      </select>

      <button on:click={handleLoadExtension} disabled={loading || !selectedExtension}>
        📥 Carregar
      </button>

      <button on:click={handleUnloadExtension} disabled={loading || !selectedExtension}>
        📤 Descarregar
      </button>
    </div>

    {#if selectedExtension && extensions.find(e => e.id === selectedExtension)}
      {@const ext = extensions.find(e => e.id === selectedExtension)}
      <div class="extension-info">
        <p><strong>Nome:</strong> {ext.name}</p>
        <p><strong>Autor:</strong> {ext.author}</p>
        <p><strong>Versão:</strong> {ext.version}</p>
        <p><strong>Tipo:</strong> {ext.extensionType}</p>
        <p><strong>Idioma:</strong> {ext.language}</p>
        <p><strong>Status:</strong> {ext.loaded ? '✓ Carregada' : '○ Não carregada'}</p>
        <p><strong>Capabilities:</strong></p>
        <ul>
          <li>Busca: {ext.exports.search ? '✓' : '✗'}</li>
          <li>Capítulos: {ext.exports.getChapters ? '✓' : '✗'}</li>
          <li>Imagens: {ext.exports.getChapterImages ? '✓' : '✗'}</li>
          <li>Multi-idioma: {ext.exports.isMultiLanguage ? '✓' : '✗'}</li>
        </ul>
      </div>
    {/if}
  </div>

  <!-- Busca -->
  <div class="section">
    <h2>2. Buscar Conteúdo</h2>
    <div class="controls">
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Digite algo para buscar..."
        disabled={loading}
        on:keypress={(e) => e.key === 'Enter' && handleSearch()}
      />
      <button on:click={handleSearch} disabled={loading || !searchQuery}>
        🔍 Buscar
      </button>
    </div>

    {#if searchResults.length > 0}
      <div class="results">
        <h3>{searchResults.length} resultados:</h3>
        <div class="favorites-grid">
          {#each searchResults as favorite}
            <div class="favorite-card" on:click={() => handleSelectFavorite(favorite)}>
              <img src={favorite.cover} alt={favorite.name} />
              <div class="favorite-info">
                <h4>{favorite.name}</h4>
                {#if favorite.extraName}
                  <p class="extra">{favorite.extraName}</p>
                {/if}
                {#if favorite.description}
                  <p class="description">{favorite.description.slice(0, 100)}...</p>
                {/if}
                <p class="source">📚 {favorite.source}</p>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>

  <!-- Capítulos -->
  {#if selectedFavorite}
    <div class="section">
      <h2>3. Capítulos de: {selectedFavorite.name}</h2>

      {#if languages.length > 0}
        <div class="controls">
          <label>
            Idioma:
            <select bind:value={selectedLanguage} on:change={handleChangeLanguage}>
              {#each languages as lang}
                <option value={lang.id}>{lang.label}</option>
              {/each}
            </select>
          </label>
        </div>
      {/if}

      {#if chapters.length > 0}
        <div class="chapters-list">
          {#each chapters as chapter}
            <div class="chapter-item" on:click={() => handleSelectChapter(chapter)}>
              <span class="chapter-number">Cap. {chapter.number}</span>
              {#if chapter.title}
                <span class="chapter-title">{chapter.title}</span>
              {/if}
              {#if chapter.scan}
                <span class="chapter-scan">{chapter.scan}</span>
              {/if}
              {#if chapter.language}
                <span class="chapter-lang">{chapter.language}</span>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  <!-- Imagens -->
  {#if selectedChapter}
    <div class="section">
      <h2>4. Imagens do Capítulo {selectedChapter.number}</h2>

      {#if images.length > 0}
        <div class="images-grid">
          {#each images as image, i}
            <div class="image-item">
              <img src={image} alt="Página {i + 1}" loading="lazy" />
              <span class="image-number">Página {i + 1}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  <!-- Logs -->
  <div class="section logs-section">
    <div class="logs-header">
      <h2>📋 Logs</h2>
      <button on:click={clearLogs}>Limpar</button>
    </div>
    <div class="logs">
      {#each logs as log}
        <div class="log-entry">{log}</div>
      {/each}
    </div>
  </div>

  {#if loading}
    <div class="loading-overlay">
      <div class="spinner"></div>
      <p>Carregando...</p>
    </div>
  {/if}
</div>

<style>
  .extension-tester {
    padding: 2rem;
    max-width: 1400px;
    margin: 0 auto;
    position: relative;
    height: 100vh;
    overflow-y: auto;
  }

  h1 {
    margin-bottom: 2rem;
    color: #333;
  }

  .section {
    background: white;
    border-radius: 8px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  h2 {
    margin-top: 0;
    margin-bottom: 1rem;
    color: #555;
  }

  .error {
    background: #fee;
    border: 1px solid #fcc;
    border-radius: 4px;
    padding: 1rem;
    margin-bottom: 1rem;
    color: #c33;
  }

  .controls {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
    flex-wrap: wrap;
  }

  select, input[type="text"] {
    flex: 1;
    min-width: 200px;
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 14px;
  }

  button {
    padding: 0.5rem 1rem;
    background: #4CAF50;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  }

  button:hover:not(:disabled) {
    background: #45a049;
  }

  button:disabled {
    background: #ccc;
    cursor: not-allowed;
  }

  .extension-info {
    background: #f9f9f9;
    padding: 1rem;
    border-radius: 4px;
    margin-top: 1rem;
  }

  .extension-info p {
    margin: 0.5rem 0;
  }

  .extension-info ul {
    margin: 0.5rem 0;
    padding-left: 1.5rem;
  }

  .favorites-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 1rem;
    margin-top: 1rem;
  }

  .favorite-card {
    border: 1px solid #ddd;
    border-radius: 8px;
    overflow: hidden;
    cursor: pointer;
    transition: transform 0.2s, box-shadow 0.2s;
  }

  .favorite-card:hover {
    transform: translateY(-4px);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
  }

  .favorite-card img {
    width: 100%;
    height: 280px;
    object-fit: cover;
  }

  .favorite-info {
    padding: 0.75rem;
  }

  .favorite-info h4 {
    margin: 0 0 0.5rem 0;
    font-size: 14px;
    line-height: 1.3;
  }

  .favorite-info .extra {
    font-size: 12px;
    color: #666;
    margin: 0.25rem 0;
  }

  .favorite-info .description {
    font-size: 11px;
    color: #888;
    margin: 0.5rem 0;
  }

  .favorite-info .source {
    font-size: 11px;
    color: #4CAF50;
    margin: 0.5rem 0 0 0;
  }

  .chapters-list {
    max-height: 400px;
    overflow-y: auto;
  }

  .chapter-item {
    padding: 0.75rem;
    border-bottom: 1px solid #eee;
    cursor: pointer;
    display: flex;
    gap: 1rem;
    align-items: center;
  }

  .chapter-item:hover {
    background: #f5f5f5;
  }

  .chapter-number {
    font-weight: bold;
    min-width: 80px;
  }

  .chapter-title {
    flex: 1;
  }

  .chapter-scan, .chapter-lang {
    font-size: 12px;
    color: #666;
    padding: 0.25rem 0.5rem;
    background: #f0f0f0;
    border-radius: 4px;
  }

  .images-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 1rem;
    margin-top: 1rem;
  }

  .image-item {
    position: relative;
  }

  .image-item img {
    width: 100%;
    height: 220px;
    object-fit: cover;
    border-radius: 4px;
    border: 1px solid #ddd;
  }

  .image-number {
    position: absolute;
    bottom: 0.5rem;
    right: 0.5rem;
    background: rgba(0, 0, 0, 0.7);
    color: white;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 12px;
  }

  .logs-section {
    background: #1e1e1e;
    color: #d4d4d4;
  }

  .logs-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .logs-header h2 {
    color: #d4d4d4;
  }

  .logs {
    max-height: 300px;
    overflow-y: auto;
    font-family: 'Courier New', monospace;
    font-size: 12px;
    line-height: 1.5;
    background: #252525;
    padding: 1rem;
    border-radius: 4px;
    margin-top: 1rem;
  }

  .log-entry {
    padding: 0.25rem 0;
    border-bottom: 1px solid #333;
  }

  .loading-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .spinner {
    border: 4px solid #f3f3f3;
    border-top: 4px solid #4CAF50;
    border-radius: 50%;
    width: 40px;
    height: 40px;
    animation: spin 1s linear infinite;
  }

  .loading-overlay p {
    color: white;
    margin-top: 1rem;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
</style>
