import itertools


class Point:
    def __init__(self, label, x, y):
        self.label = label
        self.x = x
        self.y = y
        self.color = None

    def __sub__(self, other):
        return Point(0, self.x - other.x, self.y - other.y)


class Polygon:
    def __init__(self, points):
        self.points = points

    def segments(self):
        for i in range(0, len(self.points)):
            yield (self.points[i - 1], self.points[i])

    def contains(self, point):
        return sum(
            map(lambda p: vertical_intersects(point, p[0], p[1]),
                self.segments())) % 2 == 1

    def ear_clipping(self):
        remaining_points = EarPointsList(
            [EarPoint(x) for x in range(len(self.points))])
        for i in range(len(remaining_points.points)):
            remaining_points[i].is_ear = is_ear(self, remaining_points, i)

        triangulation = Triangulation()
        print("\n Ear-Clipping")
        while True:
            next_index = next(
                (i for i, x in enumerate(remaining_points.points) if x.is_ear))

            triangulation.add((remaining_points[next_index - 1].index,
                               remaining_points[next_index + 1].index),
                              remaining_points[next_index].index)

            print(" -> adicionou diagonal {}-{} à triangulação".format(
                self.points[remaining_points[next_index - 1].index].label,
                self.points[remaining_points[next_index + 1].index].label))

            del remaining_points.points[next_index]

            remaining_points[next_index - 1].is_ear = is_ear(
                self, remaining_points, next_index - 1)
            remaining_points[next_index].is_ear = is_ear(
                self, remaining_points, next_index)

            if len(remaining_points.points) <= 3:
                triangulation.add_last(
                    (remaining_points[next_index - 1].index,
                     remaining_points[next_index + 1].index))
                return triangulation

    def colorize(self, triangulation):
        diagonal, vals = next(iter(triangulation.diagonals.items()))

        print("\n Coloração")

        self.points[diagonal[0]].color = "R"
        print(" -> coloriu ponto {} com a cor R".format(
            self.points[diagonal[0]].label))

        self.points[diagonal[1]].color = "G"
        print(" -> coloriu ponto {} com a cor G".format(
            self.points[diagonal[1]].label))

        self.points[vals[0]].color = "B"
        print(" -> coloriu ponto {} com a cor B".format(
            self.points[vals[0]].label))

        stack = [(diagonal[0], vals[0], diagonal[1]),
                 (diagonal[1], vals[0], diagonal[0]),
                 (diagonal[0], diagonal[1], vals[0])]
        while len(stack) != 0:
            d0, d1, other = stack.pop()
            if d0 > d1:
                d0, d1 = d1, d0

            vals = triangulation.diagonals.get((d0, d1))
            if vals is not None:
                new = next((x for x in vals if x != other))
                self.points[new].color = self.points[other].color
                print(" -> coloriu ponto {} com a cor {}".format(
                    self.points[new].label, self.points[new].color))

                stack.append((d0, new, d1))
                stack.append((d1, new, d0))

    def print(self, print_color):
        for p in self.points:
            print("Ponto {}: ({}, {})".format(p.label, p.x, p.y) +
                  ((" -> Cor: {}").format(p.color) if print_color else ""))


class Triangulation:
    def __init__(self):
        self.diagonals = {}

    def add(self, diagonal, point):
        d0, d1 = diagonal
        if d0 > d1:
            d0, d1 = d1, d0

        self.diagonals.setdefault((d0, d1), []).append(point)

        e0, e1 = d1, point
        if e0 > e1:
            e1, e0 = e0, e1

        if (e0, e1) in self.diagonals:
            self.diagonals[(e0, e1)].append(d0)

        e0, e1 = d0, point
        if e0 > e1:
            e1, e0 = e0, e1

        if (e0, e1) in self.diagonals:
            self.diagonals[(e0, e1)].append(d1)

    def add_last(self, diagonal):
        for d, v in self.diagonals.items():
            if len(v) == 1:
                if d[0] == diagonal[0] or d[1] == diagonal[0]:
                    v.append(diagonal[1])
                elif d[0] == diagonal[1] or d[1] == diagonal[1]:
                    v.append(diagonal[0])


class EarPointsList:
    def __init__(self, points):
        self.points = points

    def __getitem__(self, key):
        if key < 0:
            return self.points[key + len(self.points)]

        return self.points[key % len(self.points)]


class EarPoint:
    def __init__(self, index):
        self.index = index
        self.is_ear = False


def is_ear(polygon, remaining_points, index):
    previous = polygon.points[remaining_points[index - 1].index]
    point = polygon.points[remaining_points[index].index]
    next = polygon.points[remaining_points[index + 1].index]

    if turn(previous, point, next) <= 0:
        return False

    triangle = Polygon([previous, point, next])
    other_indexes = itertools.chain(
        range(0, index - 1), range(index + 2, len(remaining_points.points)))

    return all(
        map(
            lambda idx: not triangle.contains(polygon.points[remaining_points[
                idx].index]), other_indexes))


def vertical_intersects(point, p1, p2):
    if p1.x > p2.x:
        p1, p2 = p2, p1

    return point.x > p1.x and point.x <= p2.x and turn(p1, p2, point) >= 0


def turn(a, b, c):
    v1 = b - a
    v2 = c - b

    return v1.x * v2.y - v2.x * v1.y


def main():
    num = int(input())
    points = []
    for id in range(num):
        vals = input().split()
        point = Point(id + 1, float(vals[0]), float(vals[1]))
        points.append(point)

    polygon = Polygon(points)

    print("\n Entrada")
    polygon.print(False)

    triangulation = polygon.ear_clipping()
    polygon.colorize(triangulation)

    print("\n Final")
    polygon.print(True)


if __name__ == "__main__":
    main()
