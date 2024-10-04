export class Chapter {
  chNumber: number | string;
  title: string;
  chapterID: string;
  source: string;
  language: string;

  constructor(
    chNumber: number | string,
    title: string,
    chapterID: string,
    source: string,
    language: string = "default",
  ) {
    this.chNumber = chNumber;
    this.title = title;
    this.chapterID = chapterID;
    this.source = source;
    this.language = language;
  }
}