public class Inheritance {

    public static void main(String[] args) {

        Father father = new Son();
        System.out.println(father.i); //why 1?
        System.out.println(father.getI());  //2
        System.out.println(father.j);  //why 10?
        System.out.println(father.getJ()); //why 10?

        System.out.println();

        Son son = new Son();
        System.out.println(son.i);  //2
        System.out.println(son.getI()); //2
        System.out.println(son.j); //20
        System.out.println(son.getJ()); //why 10?
    }
}

class Son extends Father {

    int i = 2;
    int j = 20;

    @Override
    public int getI() {
        return i;
    }
}

class Father {

    int i = 1;
    int j = 10;

    public int getI() {
        return i;
    }

    public int getJ() {
        return j;
    }
}