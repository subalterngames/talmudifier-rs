from pathlib import Path
import re

RE_LINK = re.compile(r"\[(.*?)]\((https://|#)[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)\)", flags=re.MULTILINE)
RE_RELATIVE_LINK = re.compile(r"\[(.*?)]\(#.*?\)")
RE_MULTI_SPACE = re.compile(r"([ ]{2,})")
RE_IMAGE = re.compile(r"(!\[]\(.*?\))")


def to_daf(text: str, strip_code: bool = True) -> str:
    text = RE_LINK.sub(r"\1", text[:])
    text = RE_RELATIVE_LINK.sub(r"\1", text)
    text = RE_IMAGE.sub("", text)
    text = text.replace("\n\n", " ").replace("\n", " ").replace("\\`", "`").replace('"', '')
    text = RE_MULTI_SPACE.sub(" ", text)
    if strip_code:
        text = text.replace("`", "")
    return text


output_directory: Path = Path("../example_text").resolve()

template = Path("template.md").read_text(encoding="utf-8")

overview = Path("overview.md").read_text(encoding="utf-8")
config = Path("config.md").read_text(encoding="utf-8")
length = Path("length.md").read_text(encoding="utf-8")
fonts = Path("fonts.md").read_text(encoding="utf-8")
how = Path("how.md").read_text(encoding="utf-8")

readme = template.replace("@OVERVIEW@", overview).replace("@CONFIG@", config).replace("@LENGTH@", length).replace("@FONTS@", fonts).replace("@HOW@", how)
# README.
Path("../README.md").write_text(readme)

# README for rustdoc.
readme = RE_LINK.sub(r"\1", readme[:])
readme = RE_RELATIVE_LINK.sub(r"\1", readme)
readme = RE_IMAGE.sub("", readme)
readme = RE_MULTI_SPACE.sub(" ", readme)
Path("README_rs.md").write_text(readme)


# Center.
center = to_daf(overview)
output_directory.joinpath("center.md").write_text(center)

# Left.
left = to_daf(config) + " " + to_daf(length) + " " + to_daf(fonts)
output_directory.joinpath("left.md").write_text(left)

# Right.
right = to_daf(how, strip_code=False)
output_directory.joinpath("right.md").write_text(right)
