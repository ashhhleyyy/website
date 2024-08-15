+++
title = "Breaking Java for fun and profit (part 1)"
description = "\"OpenJDK Developers Hate This One Trick\""
+++

"OpenJDK Developers Hate This One Trick"

<dialogue character="leah" mood="happy">
    Is one of your special interests Java internals by any chance?
</dialogue>

If you've been programming Java for long enough, you've probably seen [people messing with the integer cache](https://pedrorijo.com/blog/java-integer-cache/), so you'd probably be able to explain what's going on if I showed `1 + 1` being equal to `3`, rather than, y'know `2`.

However, what about if I showed this code:
```java
public class What1 {
  public static void main(String[] args) {
    Go.befuddle();
    int a = 1;
    int b = 1;
    System.out.println("1 + 1 = " + (a + b));
  }
}
```
Which either crashes like this:
```
ash in ~/projects/what> make what1
java --add-opens=java.base/java.lang=ALL-UNNAMED What1
Exception in thread "main" java.lang.OutOfMemoryError: Overflow: String length out of range
	at java.base/java.lang.StringConcatHelper.checkOverflow(StringConcatHelper.java:46)
	at java.base/java.lang.StringConcatHelper.mixLen(StringConcatHelper.java:98)
	at What1.main(What1.java:6)
make: *** [Makefile:15: what1] Error 1
```
Or works perfectly fine on other JDK versions, like this:
```
ash in ~/projects/what> make what1
java --add-opens=java.base/java.lang=ALL-UNNAMED What1
1 + 1 = 2
```
<details>
<summary>Here's the Makefile, if you're interested</summary>

```
.PHONY: make
make: What1.class Go.class

Go.class: Go.java
	javac Go.java
What1.class: What1.java
	javac What1.java

.PHONY: clean
clean:
	rm *.class

.PHONY: what1
what1: What1.class Go.class
	java --add-opens=java.base/java.lang=ALL-UNNAMED What1
```

</details>

I haven't bisected exactly which JDK version introduced and fixed this "issue", though I know that JDK 8 does not throw this error (since string concatentaion is implemented differently), JDK 11 does, and JDK 17 again does not.

Later, we'll dig into where in the JDK this bug occurs, so we can find what the changes to fix this were, but first, you might be wondering what `Go.befuddle()` does.

## What on earth were you doing ash?

Well, I was messing with the integer cache (as you do), and I had the idea to reverse the entire array and swap negative and positive (so -128 becomes 127, and vice versa).

One short _reflection<fn content="pun very much intended"></fn>_ later, and I had the following:

```java
import java.lang.reflect.Field;

public class Go {
    public static void befuddle() {
        try {
            Class<?> c = Class.forName("java.lang.Integer$IntegerCache");
            Field f = c.getDeclaredField("cache");
            f.setAccessible(true);
            Object[] a = (Object[]) f.get(null);
            for (int i = 0; i < a.length; i++) {
                int j = a.length - i - 1;
                if (i >= j) break;
                Object temp = a[i];
                a[i] = a[j];
                a[j] = temp;
            }
        } catch(ClassNotFoundException | NoSuchFieldException | IllegalAccessException e){
            e.printStackTrace();
        }
    }
}
```

Now, you would expect doing something like this to break many things, like putting numbers into a list and then finding the product:

```java
import java.util.ArrayList;

public class What2 {
  public static void main(String[] args) {
    Go.befuddle();
    ArrayList<Integer> l = new ArrayList<>();
    l.add(1);
    l.add(2);
    l.add(3);
    int product = 1;
    for (int i : l) {
      product *= i;
    }
    System.out.println(product);
  }
}
```

Which obviously, it does:

```
ash in ~/projects/what> make what2
javac What2.java
java --add-opens=java.base/java.lang=ALL-UNNAMED What2
-24
```

<dialogue character="leah" mood="happy">
So that's -24 rather than 6, because 1, 2, 3 became -4, -3, -2?
</dialogue>

Yep! Since the array goes from -128 to 127 (on default JVM settings), it's not quite symetrical around 0.

But this doesn't answer what's going on in the first example, so let's see if we can work out why we crash on some JDKs, but not others. To start, we'll have a look at the bytecode generated by different versions of `javac`. Here's the difference between Java 8 and Java 11:

```patch
diff --git a/What1_jdk8.txt b/What1_jdk11.txt
index a059524..2816823 100644
--- a/What1_jdk8.txt
+++ b/What1_jdk11.txt
@@ -1,29 +1,23 @@
 Compiled from "What1.java"
 public class What1 {
   public What1();
     Code:
        0: aload_0
        1: invokespecial #1                  // Method java/lang/Object."<init>":()V
        4: return
 
   public static void main(java.lang.String[]);
     Code:
        0: invokestatic  #2                  // Method Go.befuddle:()V
        3: iconst_1
        4: istore_1
        5: iconst_1
        6: istore_2
        7: getstatic     #3                  // Field java/lang/System.out:Ljava/io/PrintStream;
-      10: new           #4                  // class java/lang/StringBuilder
-      13: dup
-      14: invokespecial #5                  // Method java/lang/StringBuilder."<init>":()V
-      17: ldc           #6                  // String 1 + 1 =
-      19: invokevirtual #7                  // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
-      22: iload_1
-      23: iload_2
-      24: iadd
-      25: invokevirtual #8                  // Method java/lang/StringBuilder.append:(I)Ljava/lang/StringBuilder;
-      28: invokevirtual #9                  // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
-      31: invokevirtual #10                 // Method java/io/PrintStream.println:(Ljava/lang/String;)V
-      34: return
+      10: iload_1
+      11: iload_2
+      12: iadd
+      13: invokedynamic #4,  0              // InvokeDynamic #0:makeConcatWithConstants:(I)Ljava/lang/String;
+      18: invokevirtual #5                  // Method java/io/PrintStream.println:(Ljava/lang/String;)V
+      21: return
 }
```
<dialogue character="leah" mood="confused">
    So they replaced a fairly normal <code>StringBuilder</code> with... whatever invokedynamic does?
</dialogue>

Seems like it! The older `StringBuilder`-based method seems to just append each segment together, but `invokedynamic` seems to be doing something clever elsewhere.

<dialogue character="leah" mood="happy">
    Let's figure it out!
</dialogue>

## Dynamic invokations?

So, according to the JVM 11 specification<fn id="jvm-spec" content="https://docs.oracle.com/javase/specs/jvms/se11/html/jvms-6.html"></fn>, there are five (yes!) ways to invoke another method from bytecode, and basically all of them are the same. In order, there is:

- `invokedynamic`
- `invokeinterface`
- `invokespecial`
- `invokestatic`
- `invokevirtual`

<dialogue character="leah" mood="happy">
    Obviously we're interested in `invokedynamic`, but could we double check what all the others are for quickly?
</dialogue>

Of course!

Starting with `invokevirtual` and `invokestatic`, these are the two that make the most sense right away. These two are used to implement "normal" method calls to instance methods and static methods respectively. Like most of the invoke- instructions, these take three operands, `owner_name`, `method_name` and `descriptor`.

Next up is `invokeinterface`, this is used when you call methods on a class that were defined by an interface, and in this case, `owner_name` is the name of the interface, as opposed to the class that implements the method.

`invokespecial` is basically the same as all the others, except it is emitted by the compiler to call the constructor (in bytecode, an instance method called `<init>`) after creating the object with `anew`.

<dialogue character="leah" mood="confused">
    If these all do basically the same thing, why isn't there just on instruction for "invoke a method".
</dialogue>

Well, the four instuctions all have different semantics about how they resolve what method to invoke, and I assume that the split is for both performance/optimisation and organisational reasons<fn id="citation-needed" content="citation needed"></fn>.

invokedynamic is a bit different to the others, in that rather than the operand specifying the method to invoke, it specifies another method, which will return the target for it to invoke.

We can use my [bad framework](https://git.ashhhleyyy.dev/ash/bf) to experiment a little with invokedynamic, first creating a basic class, with a method we can easily call:

```java
import dev.ashhhleyyy.bad.framework.ClassMaker;
import org.objectweb.asm.Handle;
import org.objectweb.asm.Opcodes;

public class IDTest {
    public static void main(String[] args) throws Exception {
        ClassMaker maker = new ClassMaker("dev/ashhhleyyy/runtime/RuntimeClass_");
        Class<?> cls = maker.makeClass(builder -> {
            builder.iface("dev/ashhhleyyy/wip/IDTest$IIDTest");
            builder.method(Opcodes.ACC_PUBLIC, "<init>", "()V", null, new String[0], bytecode -> {
                bytecode.aload(0);
                bytecode.invokespecial("java/lang/Object", "<init>", "()V");
                bytecode.$return();
            });
            builder.method(Opcodes.ACC_PUBLIC, "test", "()V", null, new String[0], bytecode -> {
                bytecode.$return();
            });
        });
        IIDTest test = (IIDTest) cls.getConstructor().newInstance();
        test.test();
    }

    // Target method we want to call
    public static void dynamicTarget() {
        System.out.println("i have been chosen dynamically");
    }

    // Interface so we can access the method without more reflection
    interface IIDTest {
        void test();
    }
}
```

<dialogue character="leah" mood="happy">
    Ah! Our use of an interface there is what could be described as duck typing<fn content="https://github.com/SpongePowered/Mixin/wiki/Introduction-to-Mixins---Understanding-Mixin-Architecture"></fn>!
</dialogue>

Now, how do we actually use invokedynamic? Well, let's have a look at the `bytecode.invokedynamic()` builder method<fn content="https://git.ashhhleyyy.dev/ash/bf/src/branch/main/src/main/java/dev/ashhhleyyy/bad/framework/BytecodeBuilder.java#L154"></fn>.

```java
public void invokedynamic(
    String name,
    String descriptor,
    Handle bsm,
    Object... bsmArgs
)
```

Let's grab our copy of the spec<fn id="jvm-spec"></fn> again, and see if we can work out what all these are.

>  First, the unsigned indexbyte1 and indexbyte2 are used to construct an index into the run-time constant pool of the current class (§2.6), where the value of the index is (indexbyte1 << 8) | indexbyte2. The run-time constant pool entry at the index must be a symbolic reference to a dynamically-computed call site (§5.1).

Following the reference through to section 5.1, and then scrolling down slightly, we find

> A symbolic reference to a dynamically-computed call site is derived from a CONSTANT_InvokeDynamic_info structure (§4.4.10).

So what's a `CONSTANT_InvokeDynamic_info` structure?

Following yet another reference, we get to what we're looking for-

<dialogue character="leah" mood="surprised">
    Specfications, am I right
</dialogue>

Well, these sorts of documents have to be expansive and describe every behaviour, otherwise we wouldn't have a programming language and ecosystem that runs on 3 Billion Devices(tm)<fn id="citation-needed"></fn>. 

Anyway, getting back on track, here's what we were looking for:

``` 
CONSTANT_InvokeDynamic_info {
    u1 tag;
    u2 bootstrap_method_attr_index;
    u2 name_and_type_index;
}
```

> The tag item of a CONSTANT_InvokeDynamic_info structure has the value CONSTANT_InvokeDynamic (18). 
> ...
> The value of the bootstrap_method_attr_index item must be a valid index into the bootstrap_methods array of the bootstrap method table of this class file (§4.7.23).
> ...
> The value of the name_and_type_index item must be a valid index into the constant_pool table. The constant_pool entry at that index must be a CONSTANT_NameAndType_info structure (§4.4.6). This constant_pool entry indicates a name and descriptor.
> ...
> In a CONSTANT_InvokeDynamic_info structure, the indicated descriptor must be a method descriptor (§4.3.3). 

To cut a long story short, this structure points to a "bootstrap method" (which is what the bsm in the builder method is short for). This is the difference between invokedynamic and the other invoke* instructions, rather than directly specifying the target method, we get to specify a method which will resolve the target at runtime.

<dialogue character="leah" mood="surprised">
    Finally we got there, what does that method actually have to look like?
</dialogue>

Well, it can take any number of arguments, as long as the first ones are a `MethodHandles.Lookup`, a `String` and a `MethodType`.<fn content="I worked this out by trial and error"></fn> It then returns a `CallSite`, which represents the target method to call.

Let's quickly finish off our example to check our understanding, then we can start to tackle the mysterious "bug" we started with:

<dialogue character="leah" mood="happy">
    <i>self-inflicted</i> "bug" :fingerguns:
</dialogue>

```java
// *snip*

import java.lang.invoke.*;
import java.lang.reflect.InvocationTargetException;

public class IDTest {
    public static void main(String[] args) throws NoSuchMethodException, InvocationTargetException, InstantiationException, IllegalAccessException {
        ClassMaker maker = new ClassMaker("dev/ashhhleyyy/runtime/RuntimeClass_");
        Class<?> cls = maker.makeClass(builder -> {
            builder.iface("dev/ashhhleyyy/wip/IDTest$IIDTest");
            // *snip*
            builder.method(Opcodes.ACC_PUBLIC, "test", "()V", null, new String[0], bytecode -> {
                bytecode.invokedynamic("testDynamic", "()V",
                    new Handle(
                        Opcodes.H_INVOKESTATIC,
                        "dev/ashhhleyyy/wip/IDTest",
                        "testDynamic",
                        "(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/CallSite;",
                        false
                    )
                );
                bytecode.$return();
            });
        });
        IIDTest test = (IIDTest) cls.getConstructor().newInstance();
        test.test();
    }

    public static CallSite testDynamic(
            MethodHandles.Lookup lookup,
            String name,
            MethodType methodType
    ) throws NoSuchMethodException, IllegalAccessException {
        System.out.println("we are looking for a method named `" + name + "` with the type `" + methodType + '`');
        MethodHandle target = lookup.findStatic(IDTest.class, "dynamicTarget", MethodType.methodType(void.class));
        return new ConstantCallSite(target);
    }

    // *snip*
}
```

### Oops

Now, if you've been following along with me, and haven't noticed my mistake already, this is what happens when we try to run that on the latest JDK (21):

```
Exception in thread "main" java.lang.IllegalAccessError: class dev.ashhhleyyy.wip.RuntimeClass_0 cannot access its superinterface dev.ashhhleyyy.wip.IDTest$IIDTest (dev.ashhhleyyy.wip.RuntimeClass_0 is in unnamed module of loader dev.ashhhleyyy.bad.framework.ClassMaker$JvmLoader @4cb2c100; dev.ashhhleyyy.wip.IDTest$IIDTest is in unnamed module of loader 'app')
	at java.base/java.lang.ClassLoader.defineClass1(Native Method)
	at java.base/java.lang.ClassLoader.defineClass(ClassLoader.java:1017)
	at java.base/java.lang.ClassLoader.defineClass(ClassLoader.java:879)
	at dev.ashhhleyyy.bad.framework.ClassMaker$JvmLoader.injectClass(ClassMaker.java:66)
	at dev.ashhhleyyy.bad.framework.ClassMaker.makeClass(ClassMaker.java:34)
	at dev.ashhhleyyy.wip.IDTest.main(IDTest.java:13) 
```

This took me a regrettably long time to work out, I even ended up searching for this error in the OpenJDK source code<fn content="it's actually a fairly nice codebase, and i was able to understand what was going on without too much effort, despite not being particularly familiar with C++ or hotspot's internals"></fn>, trying to figure out why my runtime class generation was failing, despite me having memories of it working before.

Turns out the error message is very unhelpful, and refers to modules, whereas the actual issue is right here, at the end of `IIDTest`:

```java
    interface IIDTest {
        void test();
    }
```

I forgot to make the interface `public` 😔.

<fedi-post data-server="fedi.shorks.gay" data-id="9wyrg2m9e95z1exg"></fedi-post>

Fixing that mistake, and finally we can see that our code works!

```
$ ./gradlew run
[...]
we are looking for a method named `testDynamic` with the type `()void`
i have been chosen dynamically
```

<dialogue character="leah" mood="happy">
    Wooooooo!
</dialogue>

---

Originally I planned to make this one blog post, but I realised that it's probably worth splitting it up into two, since we've not even started digging into the string concatenation issue. Hopefully I'll write part two soon, and you can subscribe to my [RSS](/blog.rss) or [Atom](/blog.atom) feed to get notified, or even just [follow me on the fediverse](https://fedi.shorks.gay/@ash) because I'll probably post about it there too.

If you spotted any mistakes in the article, please let me know, so I can make corrections or clarifications.

See you next time!

<footnotes></footnotes>