import { Favorite } from '~/models/favorite';
import { Chapter } from '~/models/chapter'


export interface MangaDl {
  search(query: string): Promise<Favorite[]>;
  getChapters(mangaId: string): Promise<Chapter[]>;
  getChapterImages(chapterId: string): Promise<string[]>;
}