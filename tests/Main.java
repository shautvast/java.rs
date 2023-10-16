public class Main {

    final static String a;

    static{
        a="";
    }

   public static void main(String[] args){
        FloatBean f = new FloatBean();
        f.setValue(42F);
        System.out.println(f.getValue());
   }
}
