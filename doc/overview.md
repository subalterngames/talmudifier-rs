**Generate PDFs with page layouts similar to the [Talmud](https://en.wikipedia.org/wiki/Talmud#/media/File:First_page_of_the_first_tractate_of_the_Talmud_(Daf_Beis_of_Maseches_Brachos).jpg).**

Given three paragraphs of markdown text, Talmudifier will generate a .pdf file using XeTeX (via Rust's tectonic crate). You can also include a title, basic styling (bold, italic, etc.) and marginalia. Talmudifier defaults to English, but you can set the language per-column.

![A PDF that Talmudifier generated. There are three columns of text, and of varying widths. The text is pulled from this README.](images/daf.jpg)