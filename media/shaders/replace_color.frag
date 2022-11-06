#version 100

precision lowp float;

varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;

uniform vec4 find_color_1;
uniform vec4 replace_color_1;

uniform vec4 find_color_2;
uniform vec4 replace_color_2;

uniform vec4 find_color_3;
uniform vec4 replace_color_3;

uniform vec4 find_color_4;
uniform vec4 replace_color_4;

uniform vec4 find_color_5;
uniform vec4 replace_color_5;

uniform vec4 find_color_6;
uniform vec4 replace_color_6;

void replace_color(inout vec4 base_color, in vec4 find_color, in vec4 replace_color) {
    if (find_color.a != 1.0) {
        return;
    }
    if (find_color.rgb == base_color.rgb) {
        if (replace_color.a != 1.0) {
            discard;
        }
        base_color.rgb = replace_color.rgb;
    }
}

void main() {
    vec4 base_color = texture2D(Texture, uv);

    if (base_color.a != 1.0) {
        // Keep transparent pixels transparent.
        discard;
    }
    replace_color(base_color, find_color_1, replace_color_1);
    replace_color(base_color, find_color_2, replace_color_2);
    replace_color(base_color, find_color_3, replace_color_3);
    replace_color(base_color, find_color_4, replace_color_4);
    replace_color(base_color, find_color_5, replace_color_5);
    replace_color(base_color, find_color_6, replace_color_6);

    vec3 res = base_color.rgb * color.rgb;

    gl_FragColor = vec4(res, 1.0);
}
