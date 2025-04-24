from pdf2image import convert_from_path

from pathlib import Path
import re

RE_LINK = re.compile(r"\[(.*?)]\((https://|#)[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)\)", flags=re.MULTILINE)
RE_RELATIVE_LINK = re.compile(r"\[(.*?)]\(#.*?\)")
RE_MULTI_SPACE = re.compile(r"([ ]{2,})")
RE_IMAGE = re.compile(r"[ ]{0,}(!\[(.*?)]\((images/(.*?).jpg)\))")
RE_BULLET = re.compile(r"( {0,}-)", flags=re.MULTILINE)
RE_HYPHEN = re.compile(r"[a-z](\-)[a-z]")


def to_daf(text: str, strip_code: bool = True) -> str:
    text = RE_LINK.sub(r"\1", text[:])
    text = RE_RELATIVE_LINK.sub(r"\1", text)
    text = RE_IMAGE.sub("", text)
    text = text.replace("\n\n", " ").replace("\n", " ").replace("\\`", "`").replace('"', '').replace("\\*", "")
    text = RE_MULTI_SPACE.sub(" ", text)
    text = RE_BULLET.sub(" ", text)
    text = RE_HYPHEN.sub(" ", text)
    if strip_code:
        text = text.replace("`", "")
    return text


output_directory: Path = Path("../example_text").resolve()

template = Path("template.md").read_text(encoding="utf-8")

overview = Path("overview.md").read_text(encoding="utf-8")
config = Path("config.md").read_text(encoding="utf-8")
length = Path("length.md").read_text(encoding="utf-8")
fonts = Path("fonts.md").read_text(encoding="utf-8")
markdown = Path("markdown.md").read_text(encoding="utf-8")
how = Path("how.md").read_text(encoding="utf-8")

readme = template.replace("@OVERVIEW@", overview).replace("@CONFIG@", config).replace("@LENGTH@", length).replace("@FONTS@", fonts).replace("@HOW@", how).replace("@MARKDOWN@", markdown)
# README.
Path("../README.md").write_text(readme)

# README for rustdoc.
readme = RE_LINK.sub(r"\1", readme[:])
readme = RE_RELATIVE_LINK.sub(r"\1", readme)
readme = RE_IMAGE.sub(r"\n![\2][\4]\n", readme).strip()
readme = RE_MULTI_SPACE.sub(" ", readme)
Path("README_rs.md").write_text(readme)


# Center.
center = to_daf(overview)
output_directory.joinpath("center.md").write_text(center.strip())

# Left.
left = " ".join([to_daf(config), to_daf(length), to_daf(fonts), to_daf(markdown, strip_code=False)])
output_directory.joinpath("left.md").write_text(left.strip())

# Right.
right = to_daf(how, strip_code=False)
output_directory.joinpath("right.md").write_text(right.strip())

# Regenerate the daf image.
images = convert_from_path("../out.pdf")
images[0].resize((850, 1100)).save("../images/daf.jpg")
