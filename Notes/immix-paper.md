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

? reference counting is incomplete