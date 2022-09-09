# walker

This is a simulation program for a random math problem.

## 1. Introduction & Preliminaries

**Definition 1.1.**&nbsp; Let $P=(p_k)_{k=1}^n$ denote a *polygonal path* with vertices $(p_1,p_2,\dots,p_n)$ and edges $(\overline{p_1p_2},\overline{p_2p_3},\dots,\overline{p_{n-1}p_n})$. The set of all the points on $P$, denoted $\widetilde{P}$, is the union of all its edges: $$\widetilde{P}=\bigcup_{i=1}^{n-1}\overline{p_{i}p_{i+1}}.$$

**Definition 1.2.**&nbsp; A polygonal path $(p_k)_{k=1}^n$ is *simple* if and only if all of the following conditions hold:

$$
\begin{aligned}
&1)\quad\overline{p_{i-1}p_i}\cap\overline{p_ip_{i+1}}=\{p_i\},&\forall i:1<i<n;\\
&2)\quad\overline{p_1p_2}\cap\overline{p_{n-1}p_n}=\{p_1\}\cap\{p_n\},&\text{if }\,n>3;\\
&3)\quad\overline{p_{i-1}p_i}\cap\overline{p_jp_{j+1}}=\varnothing,
\end{aligned}\\
\forall i,j:1<i<j<n,\,(i,j)\ne(2,n-1).
$$

**Definition 1.3.**&nbsp; A *step sequence* of length $n$, denoted $S_n=(\mathbf{s}_k)_{k=1}^n$, is a sequence of **non-collinear** vectors in $\mathbb{R}^2$. The *walk* of $S_n$, denoted $\mathrm{walk}(S_n)$, is the polygonal path $(p_k)_{k=0}^{n}$ where $p_0=(0,0)$ and $$p_i=p_{i-1}+\mathbf{s}_i,\quad1\le i\le n.$$

The vectors $(\mathbf{s}_k)$ are referred to as *steps* of the walk.

**Theorem 1.4.**&nbsp; For any step sequence $S_n$, there exists a permutation $S_n'$ of $S_n$ such that the walk of $S_n'$ is simple.

The proof is left as an exercise to the reader.

**Problem 1.5.**&nbsp; Define the *simpleness* of a step sequence as the number of its permutations of which the walk is simple. Find the sequence $(a_n)_{n=1}^\infty$ where $a_n$ equals the minimum simpleness of a step sequence of length $n$, and describe the properties of such a step sequence with minimum simpleness.

**Problem 1.6.**&nbsp; Find the sequence $(b_n)_{n=3}^\infty$ where $b_n$ equals the minimum simpleness of a step sequence of length $n$ in which **all the vectors sums to zero**.

**Conjecture 1.7.**&nbsp; We conjecture from observations that

$$
\begin{aligned}
(a_1,a_2,\dots,a_6)&=(1,2,4,8,28,100);\\
(b_3,b_4,\dots,b_7)&=(6,16,40,168,700);\\
b_n&=n\cdot a_{n-1},\quad\forall n\ge3.
\end{aligned}
$$

**Conjecture 1.8.**&nbsp; For a step sequence $S_n$ with
$$P=\mathrm{walk}(S_n)=(p_k)_{k=0}^{n},$$

the simpleness of $S_n$ equals $a_n$ only if $p_0\ne p_n$ and
$$\widetilde{P}\cap\overline{p_0p_n}=\{p_0,p_n\}.$$
