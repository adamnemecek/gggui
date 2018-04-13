#version 150 core

uniform sampler2D t_Color;

in vec2 v_TexCoord;
in vec4 v_Color;
flat in uint v_Mode;

out vec4 Target0;

void main() {
    // Text
    if (v_Mode == uint(0)) {
        Target0 = v_Color * vec4(1.0, 1.0, 1.0, texture(t_Color, v_TexCoord).a);

    // Image
    } else if (v_Mode == uint(1)) {
        Target0 = v_Color * texture(t_Color, v_TexCoord);

    // 2D Geometry
    } else if (v_Mode == uint(2)) {
        Target0 = v_Color;
    }
}
