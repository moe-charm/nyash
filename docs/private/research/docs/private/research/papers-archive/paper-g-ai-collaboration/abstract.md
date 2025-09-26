# Abstract

## English Version

We present a compelling case study of human-AI collaboration in compiler development where fundamental design principles overlooked by state-of-the-art AI systems were rediscovered through human intuition and implementation pain. During the development of the Nyash programming language, three leading AI assistants (ChatGPT, Claude, and Gemini) collectively failed to recognize the necessity of type information in the intermediate representation (MIR), focusing instead on instruction minimization and architectural elegance. 

This oversight led to a 650-line implementation struggle, with debugging sessions exceeding 50 minutes for simple string operations. Remarkably, the human developer—a programming novice with no formal knowledge of compiler theory or intermediate representations—independently identified the need for type information through the direct experience of implementation difficulties. The insight emerged from a simple observation: "Why doesn't the compiler know if '+' means string concatenation or numeric addition?"

Our analysis reveals three key factors in AI's oversight: (1) partial optimization bias, where AIs focused exclusively on the assigned goal of minimizing instruction count, (2) lack of implementation experience, preventing AIs from anticipating practical debugging challenges, and (3) fragmented context across multiple AI consultations. In contrast, human learning through "implementation pain" led to fundamental insights that escaped theoretical analysis.

This case study introduces "Implementation-Driven Learning" as a complementary paradigm to AI-assisted development, demonstrating that human intuition grounded in practical experience remains irreplaceable even in the age of AI. We propose a new collaboration model where AI handles theoretical optimization while humans contribute experiential learning and holistic problem identification.

## 日本語版

本研究は、最先端AIシステムが見落とした基本的な設計原理を、人間の直感と実装の苦痛を通じて再発見したコンパイラ開発における人間-AI協働の説得力のある事例を提示する。Nyashプログラミング言語の開発において、3つの主要なAIアシスタント（ChatGPT、Claude、Gemini）は、命令数の最小化とアーキテクチャの優雅さに焦点を当てる一方で、中間表現（MIR）における型情報の必要性を認識することに失敗した。

この見落としは650行に及ぶ実装の苦闘を招き、単純な文字列操作のデバッグセッションは50分を超えることもあった。注目すべきことに、コンパイラ理論や中間表現の正式な知識を持たないプログラミング初心者である人間の開発者は、実装の困難さを直接経験することで、型情報の必要性を独自に特定した。この洞察は単純な観察から生まれた：「なぜコンパイラは'+'が文字列連結なのか数値加算なのか分からないの？」

我々の分析は、AIの見落としにおける3つの主要因を明らかにした：（1）部分最適化バイアス - AIが命令数最小化という与えられた目標に専念しすぎた、（2）実装経験の欠如 - 実践的なデバッグの課題を予測できなかった、（3）複数のAI相談にわたる断片化された文脈。対照的に、「実装の苦痛」を通じた人間の学習は、理論的分析では見逃された根本的な洞察をもたらした。

この事例研究は、AI支援開発への補完的パラダイムとして「実装駆動型学習」を導入し、実践的経験に根ざした人間の直感がAI時代においても代替不可能であることを実証する。我々は、AIが理論的最適化を担当し、人間が経験的学習と全体的な問題識別に貢献する新しい協働モデルを提案する。

## Keywords / キーワード

Implementation-Driven Learning, Human-AI Collaboration, Compiler Design, Type Systems, Experiential Knowledge, Software Engineering

実装駆動型学習、人間-AI協働、コンパイラ設計、型システム、経験的知識、ソフトウェア工学