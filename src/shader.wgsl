// Shader that draws circles

struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct Dot {
    @location(1) screenPosition: vec2<f32>,
    @location(2) radius: f32,
    @location(3) hardness: f32,
    @location(4) color: vec4<f32>,
    @builtin(instance_index) instanceIndex: u32,
}

struct Uniforms {
    frame: u32,
    _padding: u32,
    _padding2: u32,
    _padding3: u32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) dot: vec2<f32>,
    @location(1) radius: f32,
    @location(2) color: vec4<f32>,
    @location(3) hardness: f32,
}


@vertex
fn vs_main(vertex: VertexInput, dot: Dot) -> VertexOutput {
    var out: VertexOutput;

    out.position = vec4<f32>((vertex.position - 0.5) * dot.radius + dot.screenPosition + sin(f32(uniforms.frame + dot.instanceIndex) / 10.0) * 0.01, 0.0, 1.0);
    out.dot =  vertex.position - 0.25;
    out.radius = dot.radius;
    out.color = dot.color;
    out.hardness = dot.hardness;

    return out;
}


@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {

    let a = input.dot - vec2(0.25, 0.25);
    let distance = dot(a, a) * 2.0;

    let circle = (1.0) - smoothstep(0.0 + input.hardness / 2.0, 0.5, distance);



    return vec4(input.color.xyz, input.color.w * circle);
}
