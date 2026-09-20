-> Paper
performance objectives: **space efficiency**, **fast reclamation**, **mutator performance**
canonical tracing collector families: **semi-space**, **mark-sweep**, **mark-compact**
new family: mark-region
opportunistic defragmentation = copy + mark in one pass


-> Definitions
- Immix = mark-region + opportunistic defragmentation
- block = coarse grain allocation units
- lines = finer grain than blocks


-> Literature
- tracing GC  = allocation + identification + reclamation
>universally identification is done by marking objects during a transitive closure over the *object graph* 

- Reclamation strategy dictates allocation strategy, the literature identifies three:
	1) sweep-to-free-list: sweep to a free list (e.g libc malloc)
	2) evacuation: see [this](https://wingolog.org/archives/2022/12/10/a-simple-semi-space-collector "https://wingolog.org/archives/2022/12/10/a-simple-semi-space-collector")
	3) compaction: relocate live objects to the start of the heap


-> Trade offs
mark-sweep: in smaller heap sizes, *space* and *collector efficiency* perform best since the overheads of garbage collection dominate total performance
semi-space: out performs other families at mutator performance because of cache locality, for large heap sizes that translates to less collection time
mark-compact: is noncompetitive in this setting due to the overwhelming collection cost


-> Understanding the trade offs
- mark-sweep: it allocate's from a *free list*, mark live objects, and then sweep-to-free-list puts memory back on the free list, because it's non moving, it's both space and time efficient, but because it doesn't provide locality for contemporaneously allocated objects
- semi-space: older-first, garbage-first, and others evacuate by moving all live objects to a new space, reclaiming the old space en masse
- Mark-compact, the compressor, and others compact by moving all live objects to one end of the same space, reclaiming the unused portion en mass
>compaction and evacuation strategies provide contiguous allocation, which puts contemporaneously allocated objects next to each other, thus offering mutator locality. However evacuation incurs 2 * space overhead and in-place compaction is time inefficient because it requires multiple passes over the heap


-> Questions
? reference counting is incomplete

# Mark Region
## A Naive Implementation

-> The Algorithm:
- the heap is divided into fixed regions
- each region is either free or unavailable
- alloactor bump allocates into free regions until all free regions are exhausted
- the collector marks any regions containing a live object as unavailable and all the other regions as free

two questions arise:
(1) How big should the regions be? big regions are space-inefficient since a single small object can withhold an entire region, while small regions increase space-efficiency but increasing collection-time
(2) How to defragment? An **entire region** is unavailable as long as any object within it remains alive, so defragmentation is essential

immix handles those two questions respectively:
(1) region sizing is fixed by operating at two levels, coarse grained blocks and fine grained lines, by **reclaiming at line granularity**, allocators skip over unavailable lines when recycling partially used blocks, objects may span lines but cannot span blocks
(2) Immix uses lightweight *opportunistic* evacutation, which is merged with marking when defragmentation is necessary. more on that later

NOTE: I'm not going to go in-depth in what the book[^1] has already explained, but I will skim through some points

-> Design
Identification: the collector performs transitive closure over the object graph, **it marks objects and lines in a line map**
Reclamation: when immix completes the sweep it performs a *coarse-grained* sweep, linearly scanning the line map and entirely free blocks and free lines in partially free blocks, it returns entirely free blocks to a global pool, and recycles partially free blocks for next allocations

-> Defragmentation
At the start of each collection, immix decides whether to defragment e.g based on fragmentation levels, if so immix chooses defragmentation **candidates** and **evacution targets** based on the previous collection's statistics

[^1]: Writing Interpreters in Rust: a Guide: [rust-hosted-langs.github.io/book](https://rust-hosted-langs.github.io/book/)
