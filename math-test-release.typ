#set page(width: 210mm, height: 297mm, margin: 20mm)
#set text(font: ("Libertinus Serif",), size: 12pt)
#show raw: set text(font: ("DejaVu Sans Mono",), size: 11.04pt)
#show link: set text(fill: blue)
#pagebreak()
= Table of Contents
- Math Support Test
  - Inline Math
  - Display Math (Block)
  - Fractions and Nested Expressions
  - Matrices
  - Cases (Piecewise Functions)
  - Operators and Relations
  - Square Roots
  - Calculus
  - Fenced Math Block
  - Text in Math
#pagebreak()

= Math Support Test

== Inline Math

The quadratic formula is $x = frac(-b plus.minus sqrt(b^2 - 4 a c), 2a)$.

Euler’s identity: $e^{i pi} + 1 = 0$.

The area of a circle is $A = pi r^2$.

Greek letters: $alpha, beta, gamma, delta, epsilon, zeta, eta, theta$.

More Greek: $lambda, mu, nu, xi, pi, rho, sigma, tau, phi.alt, psi, omega$.

Uppercase Greek: $Gamma, Delta, Lambda, Omega, Sigma, Phi$.

== Display Math (Block)

The integral of a Gaussian:

$ integral _{-oo}^{oo} e^{-x^2}   d x = sqrt(pi) $

Maxwell’s equation:

$ nabla dot.c bold(E) = frac(rho, epsilon _0) $

The Pythagorean theorem:

$ a^2 + b^2 = c^2 $

Sum notation:

$ sum _{n=1}^{oo} frac(1, n^2) = frac(pi^2, 6) $

== Fractions and Nested Expressions

$ frac(d, d x)(frac(f(x), g(x))) = frac(f'(x)g(x) - f(x)g'(x), [g(x)]^2) $

== Matrices

A 2×2 matrix:

$ mat(a, b; c, d) $

A bracket matrix:

$ mat(delim: "[", 1, 0; 0, 1) $

== Cases (Piecewise Functions)

$ f(x) = cases(x^2 &"if" x gt.eq 0, -x &"if" x < 0) $

== Operators and Relations

Inequalities: $a lt.eq b gt.eq c$ and $x eq.not y$.

Set membership: $x in RR$ and $A subset B$.

Approximation: $pi approx 3.14159$.

Arrows: $f: A ->B$ and $x |->f(x)$.

== Square Roots

$sqrt(2) approx 1.414$ and $root(3, 8) = 2$.

== Calculus

Partial derivatives: $frac(diff f, diff x)$ and $nabla f$.

A limit: $lim _{x ->0} frac(sin x, x) = 1$.

== Fenced Math Block

$ E = m c^2 $

Inline using backtick syntax: $F = m a$

== Text in Math

The velocity $v_{"max"}$ is bounded.

Using operatorname: $"sgn"(x)$.


