#version 460

layout(location = 0) flat in int VertexDrawId;

layout(location = 0) out vec4 Color;

void main()
{
	if (VertexDrawId == 0) {
		Color = vec4(1.0, 0.0, 0.0, 1.0);
	} else {
		Color = vec4(0.0, 1.0, 0.0, 1.0);
	}
}