# FINAL PLAN
## How this is different from my old plans
basically my last old plan was pretty good. But then I realized I could still encode all the data in 8 bytes instead of 16 just instead of using draw_id, use an id I embed into each instance.

## ChunkMesh
```rust
struct ChunkMesh {
    // position is send to the gpu
    offsets: [u32; 7],
}
```

```rust
enum SignedAxis {
    PosX = 0,
    PosY = 1,
    PosZ = 2,
    NegX = 3,
    NegY = 4,
    NegZ = 5,
}
```

`offsets[0]` to `offsets[1]` is `PosX`  
`offsets[1]` to `offsets[2]` is `PosY`  
`offsets[2]` to `offsets[3]` is `PosZ`  
`offsets[3]` to `offsets[4]` is `NegX`  
`offsets[4]` to `offsets[5]` is `NegY`  
`offsets[5]` to `offsets[6]` is `NegZ`

## InstanceData
```rust
#[repr(C)]
struct InstanceData(u32, u16, u16);
```

`InstanceData.0`:  
0-6: x,  
6-12: y,  
12-18: z,  
18-24: w,  
24-30: h,  
30-32: *unused*

`InstanceData.1`:  
0-3: signed axis,  
3-16: texture_index

`InstanceData.2`:  
0-16: chunk_index

## ChunkData
```rust
struct ChunkData {
    position: IVec3,
}
```

## Buffers

### Instance
Holds `InstanceData`

#### Size
Opt1: Amortized growth. Copy to a new `Buffer` with proportionally more space.  
Opt2: Chunks. We create and destroy multiple different `Buffer`s on the fly. Each `Buffer` will require its own `draw` call. Fixed size.  
**Opt3**: Mix. Chunks but they start a bit smaller than Opt2 Chunks but can grow larger. Choose a good max size so that a normal situation stays in the 1-3 chunks range.

#### Writing
Opt1: Allow the MAIN world to write to the buffer directly. This may require deffering `free` operations until the frame has ended to prevent writing data that the gpu may be reading. I am not sure if this would actually be a problem because writes are deferred anyways.  
**Opt2**: Store each write command (data + offset) in a queue which we send to the RENDER world on extraction. Allows merging of adjacent writes. Requires storing data on the cpu.  
Opt3: Persistently Mapped. Unavailable in wgpu.

### Indirect Buffer

#### Building
### Data
```rust
#[repr(C)]
struct DrawIndirect {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
}
```
`first_vertex` will always be 0  
`vertex_count` will always be 4

### Building
For every `ChunkMesh` stored on the cpu we collect every single visible slice and build a DrawIndirect for each. Then we write this buffer each frame, creating a new one if its not large enough. We will also combine adjacent slices together into one draw args, however this will usually only happen on a chunk local scale because of how the allocator works.

### Chunk Buffer
Holds a linear array of `ChunkData`. This array is allocated.

#### Size
Amortized growth. This buffer needs to be bindable to a shader storage and cannot be split.

#### Writing
Same as `InstanceData`.

---
---
---





# OLD PLAN

## How this is different from my old plans
My options for encoding global instance position are as follows:  
Opt1: Encode local_pos then add chunk_pos per draw in `multi_draw`. b/c wgpu doesnt support DrawId I would be forced to instead simply use `draw` where I give each draw a push constant. Because I separated each chunk into 6 meshes this would mean each chunk would need minimum 3 draw calls.
**Opt2**: Encode global_pos inside each instance. This doubles the required data per but allows me to still use multi_draw_indirect. This is because we effectivly decouple the instances from a chunk.

## ChunkMesh
```rust
struct ChunkMesh {
    // no longer needs a position as its embedded in each instance.
    offsets: [u32; 7],
}
```

```rust
enum SignedAxis {
    PosX = 0,
    PosY = 1,
    PosZ = 2,
    NegX = 3,
    NegY = 4,
    NegZ = 5,
}
```

`offsets[0]` to `offsets[1]` is `PosX`  
`offsets[1]` to `offsets[2]` is `PosY`  
`offsets[2]` to `offsets[3]` is `PosZ`  
`offsets[3]` to `offsets[4]` is `NegX`  
`offsets[4]` to `offsets[5]` is `NegY`  
`offsets[5]` to `offsets[6]` is `NegZ`

## Buffer Setup

### Instance Buffer
#### Data
```rust
#[repr(C)]
struct InstanceData {
    position: IVec3,
    data: u32,
}
```

`data`:  
0-16: id,  
16-22: w,  
22-28: h,  
28-31: `SignedAxis`,  
31-32: *wasted*

max abs position: ~ 2 billion

total size: 4 * 4 = 16 bytes

#### Size
Opt1: Amortized growth. Copy to a new `Buffer` with proportionally more space.  
Opt2: Chunks. We create and destroy multiple different `Buffer`s on the fly. Each `Buffer` will require its own `draw` call. Fixed size.  
**Opt3**: Mix. Chunks but they start a bit smaller than Opt2 Chunks but can grow larger. Choose a good max size so that a normal situation stays in the 1-3 chunks range.

#### Writing
Opt1: Allow the MAIN world to write to the buffer directly. This may require deffering `free` operations until the frame has ended to prevent writing data that the gpu may be reading. I am not sure if this would actually be a problem because writes are deferred anyways.  
**Opt2**: Store each write command (data + offset) in a queue which we send to the RENDER world on extraction. Allows merging of adjacent writes. Requires storing data on the cpu.  
Opt3: Persistently Mapped. Unavailable in wgpu.

### Vertex Buffer
I'm going to let bevy handle this as a `Rectange` `Mesh`.

### Indirect Buffer

#### Building
### Data
```rust
#[repr(C)]
struct DrawIndirect {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
}
```
`first_vertex` will always be 0  
`vertex_count` will always be 4

### Building
For every `ChunkMesh` stored on the cpu we collect every single visible slice and build a DrawIndirect for each. Then we write this buffer each frame, creating a new one if its not large enough. We will also combine adjacent slices together into one draw args, however this will usually only happen on a chunk local scale because of how the allocator works.

---
---
---
---
---
---

# OLD PLAN

## Pipeline sketch
1. Upon creating a mesh send its `&[InstanceData]` to the first Slab which it fits in returning relevant data (slab_index, offset). Store this referenced to the Slab along with information 
2. ... never finished because I scrapped this

## Buffer Setup

### Instance Buffer
#### Data
`struct InstanceData([u32; 2]);`

`InstanceData[0]`:  
0-6: x,  
6-12: y,  
12-18: z,  
18-24: w,  
24-30: h,  
30-32: *wasted*

`InstanceData[1]`:  
0-16: id,  
16-32: *wasted*

#### Size
Opt1: Amortized growth. Copy to a new `Buffer` with proportionally more space.  
Opt2: Chunks. We create and destroy multiple different `Buffer`s on the fly. Each `Buffer` will require its own `draw` call. Fixed size.  
**Opt3**: Mix. Chunks but they start a bit smaller than Opt2 Chunks but can grow larger. Choose a good max size so that a normal situation stays in the 1-3 chunks range.

#### Writing
Opt1: Allow the MAIN world to write to the buffer directly. This may require deffering `free` operations until the frame has ended to prevent writing data that the gpu may be reading. I am not sure if this would actually be a problem because writes are deferred anyways.  
**Opt2**: Store each write command (data + offset) in a queue which we send to the RENDER world on extraction. Allows merging of adjacent writes. Requires storing data on the cpu.  
Opt3: Persistently Mapped. Unavailable in wgpu.

### Vertex Buffer
I'm going to let bevy handle this by itself as a mesh.

### Indirect Buffer
doesnt exist

# OLD OLD PLAN

# Why it sucked
So I thought this idea was all fine and dandy, then I realized that the overhead to do 6 comparsion operations plus some frustum culling is near definately not worth a compute shaders overhead. It will probably be autovectorized on the cpu to practically nothing. This also avoids the pain I had about needing DrawId, which I can workaround now.

# Pipeline sketch
1. (`Main` world) upon generating a `ChunkMesh`, push it to a vec, then add the `QuadInstanceData` to the queue, doing any adjacency merging possible.
2. (`Extraction`) `mem::take` perhaps with `RefCell`(b/c no `mut`) the queue of data. Copy the `Vec<ChunkMesh>`.
3. (`Render` world) If changed create buffer to store the `Vec<ChunkMesh>`. Push all queue changes to the Instance buffer.
4. This is where my understaning fails me. I need to send a compute shader the `ChunkMesh` Buffer as well as some viewer information `ChunkPos`/`GlobalTransform`. It will then do simple positional comparisons to determine which chunks can be drawn. Writes those draw commands to. 

> **Idea**  
> Bevys `BufferVec` may work suitably for the `ChunkMesh`s

# Buffer Setup

## Instance
Contains a large array of `QuadInstanceData` in allocated space.

### Data
`struct QuadInstanceData([u32; 2]);`

**upper**:  
0-6: x,  
6-12: y,  
12-18: z,  
18-24: w,  
24-30: h,  
30-32: *wasted*

**lower**:  
0-16: id,  
16-32: *wasted*

### Size
We either do ***amortized growth*** or we allocate completly separate ***chunks*** which must each have their own `draw` call. A mix is probably the best solution but also the most complex.

### Writing
We can either:  
I think this would work assuming I synced the `Buffer` resource without unneccecary cloning the `Buffer` handle on extraction. ~~I dont think this would easily work because I'm pretty sure the gpu has runtime borrow checking as well as the fact that I'll need access in both the Render and Main worlds.~~ Write anytime and allow the allocator to ensure borrow safety. This requires freeing data to be deferred by a frame.  
**Queue Writes. We can store each buffer write (and merge adjacent ones) then once every frame send the queue to the RenderWorld which writes them to a buffer.**

I think that queueing and merging writes is preferable. Many writes one after eachother will often be to adjacent memory, especially with large writes or large frees, which merging into fewer write calls should be worth storing the data on the cpu. It also means that I can easily handle syncronization between Main and Render worlds by just sending the queue every frame.

### Freeing
Same as normal Allocators, assumes the data at that location will no longer be used and is available to write.

## Indirect 
Contains a large array of `DrawIndirect`.

### Data
```rust
#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
struct DrawIndirect {
    vertex_count: u32,
    instance_count: u32,
    first_vertex: u32,
    first_instance: u32,
}
```

### Size
We can either:  
~~Have a large buffer that is theoretically impossible to run out of space in.~~  
Have a large buffer with ***amortized growth***. The **minimum** size of the buffer each frame must be the same length as the number of instances in the instance buffer. This data can be extracted from the allocatiors.

### Usage
This buffer will be used in the final `multi_draw_indirect_count`.

### Writing
The plan is for the gpu to write this buffer every frame from the `Chunk` buffer in a **compute shader** which can do culling and such.

## Indirect Count
Contains a single `u32` indicating the number of elements in the `Indirect` buffer as built by the compute shader.

### Writing
**compute shader**

## ChunkMeshs 
Contains a large array of `ChunkMesh`s.

### Data
```rust
#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
struct ChunkMesh {
	position: IVec3,
	offsets: [u32; 7],
}
```

Total data: 4 * 3 + 4 * 7 = 40 bytes

Each chunk owns a large slice of the instance buffer from offsets[0] to offsets[6]. For any offset n offset[n] >= offset[n - 1].

offsets[0] to offsets[1] is PosX  
offsets[1] to offsets[2] is NegX  
offsets[2] to offsets[3] is PosY  
offsets[3] to offsets[4] is NegY  
offsets[4] to offsets[5] is PosZ  
offsets[5] to offsets[6] is NegZ

### Culling
Only certain axis need to be drawn for each chunk. The **compute shaders** task is to iterate over every `ChunkMesh`, figure out which sub slices need to be drawn, and writing their draw commands to the indirect buffer.

### Writing
This will nearly always change every frame. The compute shader also needs to have a way to iterate over every single `ChunkMesh`.

We can either:  
**Store the data cpu side and copy to a new buffer each frame.**  
**~~Allocate a large **amortized** buffer which we index into. Every frame we write an entirely new buffer containing a list of occupied indices.~~

I think that handling Allocated space may cause some overhead and especially complexity we dont need. Copying this data to a buffer every frame may be cheap enough.
