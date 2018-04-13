#version 150 core

in vec2 a_Pos;
in vec2 a_TexCoord;
in vec4 a_Color;
in uint a_Mode;

out vec2 v_TexCoord;
out vec4 v_Color;
flat out uint v_Mode;

void main() {
    v_TexCoord = a_TexCoord;
    v_Color = a_Color;
    gl_Position = vec4(a_Pos, 0.0, 1.0);
    v_Mode = a_Mode;
}