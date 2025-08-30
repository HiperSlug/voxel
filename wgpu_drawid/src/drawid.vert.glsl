#version 460

layout(location = 0) in vec2 Position;

layout(location = 0) flat out int VertexDrawId;

void main() {
	gl_Position = vec4(Position, 0.0, 1.0);
	VertexDrawId = gl_DrawID;
}