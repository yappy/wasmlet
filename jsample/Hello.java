public class Hello {
    private static native void println(String msg);

    public static void main(String[] args) {
        println("Hello, world!");
        for (int i = 0; i < args.length; i++) {
            println(args[i]);
        }
    }
}
