package dummy;

public class Dummy {

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
