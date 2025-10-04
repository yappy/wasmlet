# Technical Note

## Special Method

* `<init>`
  * 要はコンストラクタ。
  * 内部正式名は instance initialization method。
  * Java 言語上有効な識別子ではない。
    コンパイラがコンストラクタに対してこの名前を付ける。
* `<clinit>`
  * `class A { static {...} }` のように書かれるクラスイニシャライザ。
  * 正式名は class or interface initialization method。
  * JVM 命令から明示的に呼び出されることはない。
  * クラスの初期化時に暗黙に呼び出す必要がある。

## ACC_SUPER

<https://qiita.com/YujiSoftware/items/29a23aba803cb1726ac3>

`invokevirtual` はインスタンスメソッド (非 static メソッド) の通常の仮想呼び出し。
オーバーライドを考慮して実際に呼び出されるメソッドを動的に解決する必要がある。
Java 言語の文法に virtual (C++ 仮想関数) は存在せず、予約語でもないが、
ほとんどのインスタンスメソッドは実質的に C++ 仮想関数となる。

それに対して `invokespecial` はインスタンスメソッド呼び出しのうち動的解決の必要のない
特殊なもの、具体的には

* `super.method()` のようなスーパークラスメソッドの呼び出し
* private メソッドの呼び出し
* コンストラクタ呼び出し

これらは実際に呼び出すクラスがコンパイル時に決定できるので Constant Pool 中の
`methodref = class + name + type`
をそのまま使って呼び出せば OK…、と見せかけて、
スーパークラスのメソッド呼び出しは実行時に動的判定しないと事故ることが
Java 1.0.2 リリース後に判明したようだ。

* `class A`
* `class B extends A`
* `class C extends B`
* A には `public void method()` が定義されているとする。
* B にその名前のオーバーライドはない
* C で `method()` をオーバーライドし、中で `super.method()` を呼び出す。
* コンパイラはこれを静的解決し、C のコンパイル時に `class A - method()` への
  invokespecial を生成する。

これはどう考えてもダメで、B を書き換えて `method()` をオーバーライドする
ようにし B のみリコンパイルすると、C からの `super.method()` は
B の同名メソッドを呼ぶようにならなければならない。

というわけで互換性のため、Java 1.1 以降のコンパイラでは必ず `ACC_SUPER` フラグを
セットするようになり、invokevirtual のような動的解決を行うようになったそうな。

Java 7 での JVM 仕様と訳

* Next, the resolved method is selected for invocation
  unless all of the following conditions are true:
* 次に、以下のすべての条件が満たされない限りは解決されたメソッドをそのまま選択する
  (以下の3つの条件がすべて満たされるなら動的解決を行う。設計ミスの尻ぬぐい。
  どれか1つでも満たされないならそれは行わず Constant Pool の内容をそのまま使う。
  本来の設計意図。
  unless all... やめろ。)
  * The ACC_SUPER flag (Table 4.1) is set for the current class.
    * ACC_SUPER が現在のクラスに設定されていること。
  * The class of the resolved method is a superclass of the current class.
    * メソッドのクラスが現在のクラスのスーパークラスのうちの1つであること。
  * The resolved method is not an instance initialization method (§2.9).
    * メソッドがコンストラクタ (`<init>`) でないこと。

なお旧仕様を悪用し、継承途中のオーバーライドをバイパスして先の先祖のメソッドを
直接呼ぶセキュリティアタックが行われたためか、Java 7 Update 13 から
ACC_SUPER フラグの有無は見ずに強制的にフラグがセットされているものとして
処理されるようになったらしい。よく勉強して考えるね。

動的解決することになった場合の手続き。
現在のクラスの直接(1つ上)のスーパークラスを C とおく。

* もし C が name + desc に一致するメソッドを持っている場合、それを呼ぶ。
  手続きはここで完了する。
* そうではなく、かつもし C がスーパークラスを持っている場合、
  その直接のスーパークラスに対して再帰的に手続きを行う。
* そうではない場合
  (スーパークラスを持たない、つまり `Object` まで探しても見つからなかった場合)、
  `AbstractMethodError` を発生させる。
