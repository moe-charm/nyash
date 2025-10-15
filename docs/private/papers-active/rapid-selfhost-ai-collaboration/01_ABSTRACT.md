# Abstract (Draft)

## Version 1 (Technical Focus)

We present Hakorune, a self-hosting compiler developed in 63 days through collaboration between one human developer and two AI assistants (ChatGPT and Claude Code). This represents a 10-50x speedup compared to traditional language implementations (Rust: 5 years, Go: 2 years, C: 4 years). Our approach achieved 8 million lines of code changes at 133,333 lines/day, demonstrating that AI-assisted development can dramatically accelerate compiler implementation while maintaining quality (170/185 tests passing, 91.9%).

Notably, the developer—self-described as "just an ordinary person" without formal compiler expertise—independently rediscovered industry-standard patterns such as the "frozen toolchain" approach (equivalent to Rust's stage0 and Go's frozen Go 1.4), providing strong evidence for the correctness and naturalness of these design decisions. We analyze the collaborative workflow, quantify AI contributions (ChatGPT: 40% design, Claude Code: 50% implementation, Human: 10% strategic decisions), and demonstrate how "playful deepening"—an enjoyment-driven development paradigm—sustains high-speed iteration without burnout.

Our findings suggest that AI collaboration can democratize compiler development, enabling non-experts to achieve expert-level results through intuitive problem-solving rather than extensive formal training.

**Keywords**: Self-hosting compilers, AI-assisted programming, Human-AI collaboration, Rapid prototyping, Language implementation

---

## Version 2 (Human Story Focus)

Can an "ordinary person" build a self-hosting compiler in 63 days? We show that with AI collaboration, the answer is yes—and the result is 10-50x faster than traditional approaches. Hakorune, a self-hosting compiler developed through collaboration between one human and two AI assistants (ChatGPT and Claude Code), achieved full self-hosting in just 63 days, compared to 2-5 years for languages like Rust and Go.

Remarkably, the developer independently rediscovered industry-standard patterns (frozen toolchain, single-parser maintenance) without prior knowledge of how Rust, Go, or OCaml achieved self-hosting. This convergence provides strong evidence that practical constraints naturally lead to similar solutions, regardless of formal training.

We analyze the development process quantitatively (8M lines changed, 133,333 lines/day), examine the AI collaboration workflow (design vs implementation vs decision-making), and introduce "playful deepening"—a sustainable development paradigm where enjoyment drives acceleration. Our work demonstrates that AI collaboration can democratize expert-level software engineering, making previously inaccessible achievements attainable for broader audiences.

**Keywords**: Human-AI collaboration, Self-hosting compilers, Democratization of expertise, Rapid language development, Enjoyment-driven development

---

## Version 3 (Academic Balance)

Self-hosting compiler development traditionally requires 2-5 years and deep expertise in language implementation. We demonstrate that human-AI collaboration can reduce this to 63 days while preserving code quality and independently arriving at industry-standard design patterns.

We present Hakorune, a self-hosting language implemented through collaboration between one developer ("just an ordinary person" by self-description) and two AI assistants (ChatGPT for design, Claude Code for implementation). The project achieved:
- Full self-hosting in 63 days (10-50x faster than Rust/Go/C)
- 8 million lines of code changes (133,333 lines/day)
- 91.9% test pass rate (170/185 tests)
- Independent rediscovery of frozen toolchain pattern (Rust stage0, Go 1.4)

We contribute: (1) quantitative analysis of AI collaboration in compiler development, (2) evidence that practical constraints lead to convergent design patterns regardless of formal training, (3) identification of "playful deepening" as a sustainable high-speed development paradigm, and (4) demonstration that AI collaboration can democratize expert-level achievements.

Our findings suggest that the bottleneck in compiler development is shifting from implementation speed to strategic decision-making, and that AI collaboration enables rapid exploration of the design space while maintaining quality.

**Keywords**: Self-hosting compilers, AI-assisted programming, Design pattern convergence, Rapid prototyping, Software engineering productivity
