**So you wanted to build a JVM**

_as in why not???_

actually:
`System.out.println("Hello World")` would actually be a major achievement. It's nowhere near that level...

**so far**
* starts a main class (TODO cmdline args)
* loads classes from a classpath, including jar/jmod files
* instantiates classes (TODO implement superclass instantiation)
* runs bytecode (TODO more opcodes)
* has INVOKEVIRTUAL and INVOKESPECIAL, including stackframes (TODO more invokes)
* has a heap

**more TODO's**
* native methods 
* stacktraces
* check visibility
* IO
* garbage collection

**Ultimate goal** 
* Hello world domination