from math import gcd, pow
from concurrent.futures import ThreadPoolExecutor
from multiprocessing import Pool
from typing import Callable, Any, Tuple
import time
import matplotlib.pyplot as plt
import sys

sys.set_int_max_str_digits(0)

num_arctan_terms = 0


class RationalDecimal:
    __slots__ = ("n", "d")

    ZERO: "RationalDecimal"
    ONE: "RationalDecimal"

    def __init__(self, numerator, denominator=1):
        if denominator == 0:
            raise ZeroDivisionError("Denominator cannot be zero.")

        if denominator < 0:
            numerator = -numerator
            denominator = -denominator

        numerator = int(numerator)
        denominator = int(denominator)

        g = gcd(numerator, denominator)
        self.n = numerator // g
        self.d = denominator // g

    def digit_stream(self):
        integer_part = self.n // self.d
        yield str(integer_part)
        yield "."

        remainder = self.n % self.d
        seen = {}

        position = 0

        while remainder != 0:
            if remainder in seen:
                return

            seen[remainder] = position
            remainder *= 10
            digit = remainder // self.d
            yield str(digit)

            remainder %= self.d
            position += 1

    def to_decimal(self, digits: int):
        gen = self.digit_stream()
        result = []
        count = 0

        for item in gen:
            result.append(item)
            if item != ".":
                count += 1
            if count > digits:
                break

        return "".join(result)

    def __add__(self, other):
        if isinstance(other, RationalDecimal):
            return RationalDecimal(
                self.n * other.d + other.n * self.d, self.d * other.d
            )
        elif isinstance(other, int):
            return RationalDecimal(self.n + other * self.d, self.d)
        else:
            return NotImplemented

    def __radd__(self, other):
        if other == 0:
            return self
        return self.__add__(other)

    def __neg__(self):
        return RationalDecimal(-self.n, self.d)

    def __sub__(self, other):
        return self.__add__(-other)

    def __mul__(self, other):
        if isinstance(other, RationalDecimal):
            return RationalDecimal(self.n * other.n, self.d * other.d)
        elif isinstance(other, int):
            return RationalDecimal(self.n * other, self.d)
        else:
            return NotImplemented

    def __rmul__(self, other):
        return self.__mul__(other)

    def __repr__(self):
        return f"{self.n} / {self.d}"


RationalDecimal.ZERO = RationalDecimal(0, 1)
RationalDecimal.ONE = RationalDecimal(1, 1)


def rarctan(x: int, terms: int) -> RationalDecimal:
    """
    Calculates arctan(1/x) using Taylor expansion.
    """

    result = RationalDecimal.ZERO
    power = x
    sign = 1

    for i in range(terms):
        denom = power * (2 * i + 1)
        term = RationalDecimal(sign, denom)
        result += term

        power *= x * x
        sign = -sign

    return result


def compute_term(args: tuple[int, int, int]) -> RationalDecimal:
    coeff, arctan, terms = args
    return coeff * rarctan(arctan, terms)


def profile_func(func: Callable[..., Any], *args, **kwargs) -> Tuple[float, Any]:
    start = time.perf_counter()
    result = func(*args, **kwargs)
    end = time.perf_counter() - start

    return end, result


def pi_parallel(arctan_terms: int) -> RationalDecimal:
    coeffs = [
        (6348, 2852),
        (1180, 4193),
        (2372, 4246),
        (1436, 39307),
        (1924, 55603),
        (2500, 211050),
        (-2832, 390112),
    ]

    with Pool() as p:
        terms = p.map(compute_term, [(c, a, arctan_terms) for c, a in coeffs])

    res = sum(terms, RationalDecimal(0))
    return res


def pi_sequential(arctan_terms: int):
    coeffs = [
        (6348, 2852),
        (1180, 4193),
        (2372, 4246),
        (1436, 39307),
        (1924, 55603),
        (2500, 211050),
        (-2832, 390112),
    ]

    pi = RationalDecimal.ZERO

    for c, a in coeffs:
        pi += compute_term(args=(c, a, arctan_terms))

    return pi.to_decimal(arctan_terms * 7)


from math import isqrt

# Constants
A = 13591409
B = 545140134
C = 640320
C3_OVER_24 = (C**3) // 24


def bs(a, b):
    if b - a == 1:
        if a == 0:
            P = 1
            Q = 1
        else:
            P = (6 * a - 5) * (2 * a - 1) * (6 * a - 1)
            Q = a * a * a * C3_OVER_24

        T = P * (A + B * a)
        if a % 2:
            T = -T

        return P, Q, T

    m = (a + b) // 2
    P1, Q1, T1 = bs(a, m)
    P2, Q2, T2 = bs(m, b)

    P = P1 * P2
    Q = Q1 * Q2
    T = Q2 * T1 + P1 * T2

    return P, Q, T


def chudnovsky_pi(digits):
    extra = 10
    n = digits // 14 + 1

    _, Q, T = bs(0, n)

    scale = 10 ** (digits + extra)
    sqrt_term = isqrt(10005 * scale * scale)

    pi = (426880 * sqrt_term * Q) // T

    return str(pi)[:-extra]


def main():
    arctan_terms = 0
    with open("pi_1M.txt") as f:
        pi_real = f.read()

    sequential_times = []
    parallel_times = []
    chudnovsky_times = []
    precision = []

    # for i in range(101):
    #     arctan_terms += 1
    #     print(f"{arctan_terms} terms: ", end="")

    #     t_seq, pi_calc = profile_func(pi_sequential, arctan_terms)
    #     t_par, pi_calc = profile_func(pi_parallel, arctan_terms)

    #     pi_calc_gen = pi_calc.digit_stream()

    #     for j, digit in enumerate(pi_calc_gen):
    #         if pi_real[j] != digit:
    #             print(j - 2)  # ignoring 3 and "."
    #             break
    #     else:
    #         print("All correct")

    #     sequential_times.append(t_seq)
    #     parallel_times.append(t_par)
    #     precision.append(j - 2)

    # plt.plot(sequential_times, label="Sequential")
    # plt.plot(parallel_times, label="Parallel")

    for i in range(10001):
        pi_calc = str(chudnovsky_pi(i))

        print(f"{i}: ", end="")

        for j, digit in enumerate(pi_calc):
            if pi_real[j] != digit:
                print(j - 1)
                break
        else:
            print("All correct")

    # plt.plot(chudnovsky_times, label="Chudnovsky")

    # plt.legend()
    # plt.show()


if __name__ == "__main__":
    # main()
    # with open("pi_10M.txt") as f:
    #     pi_real = f.read()
    # print("Read file")
    pi_calc = profile_func(chudnovsky_pi, 10_000_000)
    # with open("pi_1B_calc.txt", "w") as f:
    #     f.write(pi_calc)

    with open("pi_1B.txt", "r") as f:
        pi_real = f.read()

    for idx, d in enumerate(pi_calc):
        if pi_real[idx] != d:
            print(f"Wrong at index: {idx}")
            break
    print("All correct")
