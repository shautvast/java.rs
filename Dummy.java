package dummy;

public class Dummy {

    private final static String constant = "meh";
    private final int integer = 57;

    private final String name;

    public Dummy(String name) {
        this.name = name;
    }

    public String getName() {
        return name;
    }

    public void print(){
        System.out.println(name);
    }

}
