#!/usr/bin/env python3
from pathlib import Path

from PIL import Image

SOURCE = Path("images/dark background.png")
OUT_DIR = Path("src-tauri/icons")
SIZE = 32
BACKGROUND = (30, 33, 38)
FUZZ = 53.0
MIN_COVERAGE = 0.15
STATES = {
    "idle": (0, 217, 200),
    "working": (75, 142, 247),
    "error": (229, 83, 75),
    "offline": (154, 162, 173),
}


def color_distance(a, b):
    return sum((a[i] - b[i]) ** 2 for i in range(3)) ** 0.5


def remove_background(image):
    pixels = image.load()
    for y in range(image.height):
        for x in range(image.width):
            if color_distance(pixels[x, y], BACKGROUND) <= FUZZ:
                pixels[x, y] = (0, 0, 0, 0)
    return image


def glyph_box(image):
    return image.getbbox()


def coverage(image):
    pixels = image.load()
    visible = sum(
        1
        for y in range(image.height)
        for x in range(image.width)
        if pixels[x, y][3] > 0
    )
    return visible / (image.width * image.height)


def has_interior_hole(image, box):
    pixels = image.load()
    left, top, right, bottom = box
    for y in range(top + 1, bottom):
        for x in range(left + 1, right):
            if pixels[x, y][3] == 0:
                return True
    return False


def recolor(image, color):
    pixels = image.load()
    for y in range(image.height):
        for x in range(image.width):
            r, g, b, a = pixels[x, y]
            if a > 0:
                pixels[x, y] = (color[0], color[1], color[2], a)


def verify(path, color):
    icon = Image.open(path)
    assert icon.size == (SIZE, SIZE), f"{path}: size {icon.size}"
    assert icon.mode == "RGBA", f"{path}: mode {icon.mode}"
    pixels = icon.load()
    assert pixels[0, 0][3] == 0, f"{path}: corner not transparent"
    solid = [(x, y) for y in range(SIZE) for x in range(SIZE) if pixels[x, y][3] == 255]
    assert solid, f"{path}: no solid glyph pixel"
    sample = pixels[solid[len(solid) // 2]]
    assert sample[:3] == color, f"{path}: color {sample[:3]} != {color}"


def main():
    glyph = remove_background(Image.open(SOURCE).convert("RGBA"))
    box = glyph_box(glyph)
    assert box is not None, "glyph not found"
    assert coverage(glyph) > MIN_COVERAGE, "glyph coverage too low"
    assert has_interior_hole(glyph, box), "knocked-out dot not found"
    base = glyph.resize((SIZE, SIZE), Image.LANCZOS)

    for state, color in STATES.items():
        icon = base.copy()
        recolor(icon, color)
        path = OUT_DIR / f"tray-{state}.png"
        icon.save(path)
        verify(path, color)
        print(f"{path} ok")


if __name__ == "__main__":
    main()
