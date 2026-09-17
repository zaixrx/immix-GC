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
	1) sweep-to-free-list
	2) evacuation
	3) compaction>

-> Trade offs
mark-sweep: in smaller heap sizes, *space* and *collector efficiency* perform best since the overheads of garbage collection dominate total performance
semi-space: out performs other families at mutator performance because of cache locality, for large heap sizes that translates to less collection time
mark-compact: is noncompetitive in this setting due to the overwhelming collection cost

-> Understanding the trade offs
- mark-sweep: it allocate's from a *free list*, mark live objects, and then sweep-to-free-list puts memory back on the free list, because it's non moving, it's both space and time efficient, but because it doesn't provide locality for contemporaneously allocated objects
- semi-space: 

? reference counting is incomplete