package com.github.shautvast.reflective;

public class MetaField {

    private final String name;
    private final int modifiers;

    public MetaField(String name, int modifiers) {
        this.name = name;
        this.modifiers = modifiers;
    }

    public String getName() {
        return name;
    }

    public int getModifiers() {
        return modifiers;
    }
}
