export const javaSample = `/**
 * Sample Java code for Code Beautifier
 * A simple Java program demonstrating basic OOP concepts
 */
package com.example.beautifier;

import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;

public class CodeSample {
    private String name;
    private List<String> items;
    
    public CodeSample(String name) {
        this.name = name;
        this.items = new ArrayList<>();
    }
    
    public void addItem(String item) {
        items.add(item);
    }
    
    public List<String> getFilteredItems(String prefix) {
        return items.stream()
                .filter(item -> item.startsWith(prefix))
                .sorted()
                .collect(Collectors.toList());
    }
    
    @Override
    public String toString() {
        return "CodeSample{" +
                "name='" + name + '\'' +
                ", items=" + items +
                '}';
    }
    
    public static void main(String[] args) {
        CodeSample sample = new CodeSample("Java Example");
        
        // Add some sample items
        sample.addItem("format");
        sample.addItem("beautify");
        sample.addItem("indent");
        sample.addItem("format code");
        sample.addItem("fix spacing");
        
        // Display filtered items
        System.out.println("Items starting with 'f':");
        List<String> filtered = sample.getFilteredItems("f");
        filtered.forEach(System.out::println);
        
        // Print the sample object
        System.out.println("\\nFull sample: " + sample);
    }
}`; 