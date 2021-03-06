#!/usr/bin/env python3

import io
import json
import math
import matplotlib.pyplot as plt
import numpy
import os
import subprocess
import time
import typing
import zlib

class Timer:
    depth = 0

    def __init__(self, name: str):
        self.name = name

    def __enter__(self):
        self.start = time.monotonic()
        self.depth = Timer.depth
        print("    " * self.depth + f"Start {self.name}")
        Timer.depth += 1
        return self

    def __exit__(self, exc_type, exc_value, tb):
        duration = time.monotonic() - self.start
        print("    " * self.depth + f"End {self.name}, took {round(duration * 1000)}ms")
        Timer.depth -= 1
        return False

class SvgPool:
    def __init__(self):
        self.pool = {}

    def read(self, path: str, size: int) -> numpy.ndarray:
        """Reads an SVG image and converts it to a numpy array of size*size"""

        with open(path, "rb") as fh:
            crc = 0
            while True:
                chunk = fh.read(4096)
                if chunk == b"":
                    break
                crc = zlib.crc32(chunk, crc)

        if crc in self.pool:
            return self.pool[crc]

        with Timer(f"converting {path} to PNG"):
            cmd = ["node", "node_modules/.bin/svgexport", path, "/dev/stdout", f"{size}:{size}"]
            buf = subprocess.check_output(cmd)
        ret = plt.imread(io.BytesIO(buf), format="png")
        self.pool[crc] = ret

        return ret

def least_power_of_two_gte(x: int) -> int:
    """Returns the smallest power of 2 greater than or equal to x"""
    y = 1
    while y < x:
        y <<= 1
    return y

class Atlas:
    def __init__(self, width: int, height: int):
        self.images = numpy.empty((width, height, 4, 1))
        self.count = 0

    def allocate(self) -> typing.Tuple[int, numpy.ndarray]:
        """Allocates a new slice in the atlas"""
        width, height, num_channels, capacity = self.images.shape
        if self.count > capacity:
            raise Exception("Index out of bounds???")
        if self.count == capacity:
            new = numpy.empty((width, height, num_channels, capacity * 2))
            new[:, :, :, :capacity] = self.images
            self.images = new

        count = self.count
        self.count += 1

        ret = self.images[:, :, :, count]
        return count, ret

    def put(self, im: numpy.ndarray) -> int:
        """Adds a new image with duplicate checking"""
        width, height, num_channels = im.shape

        with Timer("checking for duplicates"):
            # Check if there is any x such that self.images[:, :, :, x] == im is all true

            im_cmp = im.reshape((width, height, num_channels, 1))
            equal = numpy.equal(self.images[:, :, :, :self.count], im_cmp)
            equal = numpy.all(equal, axis=(0, 1, 2))

            equal_index = numpy.where(equal)[0]
            if equal_index.shape[0] > 0:
                return equal_index[0]

        with Timer("allocating new sprite"):
            index, region = self.allocate()
            region[:] = im
        return index

    def finalize(self) -> typing.Tuple[typing.List[typing.Tuple[int, int]], numpy.ndarray]:
        with Timer("Copying all sprites to the same image"):
            width, height, num_channels, capacity = self.images.shape

            dim = max(width, height) * math.ceil(math.sqrt(self.count))
            dim = least_power_of_two_gte(dim)

            output = numpy.zeros((dim, dim, num_channels))

            x = 0
            y = 0
            locs = []
            for image_ord in range(self.count):
                x2 = x + width
                if x2 > dim:
                    x = 0
                    x2 = width
                    y += height
                y2 = y + height

                output[x:x2, y:y2, :] = self.images[:, :, :, image_ord]
                locs.append((x, y))

                x = x2

            return locs, output

class Index:
    def __init__(self, size: int):
        self.svg_cache = SvgPool()
        self.atlas = Atlas(size, size)
        self.index: typing.List[typing.Tuple[str, dict]] = []

    def add_dir(self, path: str, size: int):
        subindex = {"shape": "cube"}

        for direction in ["x", "y", "z"]:
            for side in ["p", "n"]:
                im = self.svg_cache.read(os.path.join(path, direction + side + ".svg"), size)
                image_ord = self.atlas.put(im)
                subindex[direction + side] = {
                    "x": image_ord,
                    "y": image_ord,
                    "width": size,
                    "height": size,
                }

        self.index.append((os.path.basename(path), subindex))

    def add_file(self, path: str, size: int):
        im = self.svg_cache.read(path, size)
        image_ord = self.atlas.put(im)

        self.index.append((os.path.basename(path)[:-4], {
            "shape": "icon",
            "x": image_ord,
            "y": image_ord,
            "width": size,
            "height": size,
        }))

    def finalize(self):
        locs, im = self.atlas.finalize()
        output = {}
        for name, dic in self.index:
            dic = dic.copy()
            if dic["shape"] == "cube":
                for direction in ["x", "y", "z"]:
                    for side in ["p", "n"]:
                        d = dic[direction + side]

                        # For some reason, matplotlib always flips the two coordinates.
                        d["x"] = locs[d["x"]][1]
                        d["y"] = locs[d["y"]][0]
            elif dic["shape"] == "icon":
                dic["x"] = locs[dic["x"]][0]
                dic["y"] = locs[dic["y"]][1]
            output[name] = dic

        return output, im

if not os.path.isdir("../gen"):
    os.mkdir("../gen")

def main():
    sizes = {
        "pixel": 16,
        "simple": 64,
        "fancy": 256,
        "x-fancy": 1024,
    }
    for name, size in sizes.items():
        with Timer(f"creating {size}x{size} atlas"):
            index = Index(size)

            for path in os.listdir("."):
                if os.path.isfile(os.path.join(path, "xp.svg")):
                    with Timer(f"adding directory {path}"):
                        index.add_dir(path, size)
                elif path.endswith(".svg") and os.path.isfile(path):
                    with Timer(f"adding file {path}"):
                        index.add_file(path, size)

            index, im = index.finalize()

            with Timer(f"writing atlas"):
                plt.imsave(f"../gen/textures-{name}.png", im.copy(order="C"))
                with open(f"../gen/textures-{name}.png.json", "w") as fh:
                    fh.write(json.dumps(index, separators=(",", ":"), indent=1))

if __name__ == "__main__":
    with Timer("combining sprites into atlas"):
        main()
