// Fibonacci benchmark
public class Fib {
    static long fib(int n) {
        if (n <= 1) return n;
        return fib(n - 1) + fib(n - 2);
    }

    public static void main(String[] args) {
        long result = fib(40);
        System.out.println(result);
    }
}
