#version 100

precision lowp float;

varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;

uniform sampler2D color_replacement_texture;

const int MAX_COLOR_REPLACEMENTS = 8;

void replace_color(int index, inout vec4 base_color) {
    float base_pixel = float(index) * 2.0;
    float black = vec4(0.0, 0.0, 0.0, 1.0);

    vec4 find_color = texture2D(color_replacement_texture, vec2(base_pixel / 16.0, 0.0));
    vec4 replace_color = texture2D(color_replacement_texture, vec2((base_pixel + 1.0) / 16.0, 0.0));

    if (find_color.rgb == base_color.rgb) {
        if (replace_color.a != 1.0) {
            discard;
        }
        base_color = replace_color;
    }
}

void main() {
    vec4 base_color = texture2D(Texture, uv);

    if (base_color.a != 1.0) {
        // Keep transparent pixels transparent.
        discard;
    }

    for (int i = 0; i < MAX_COLOR_REPLACEMENTS; i++) {
        replace_color(i, base_color);
    }

    vec3 res = base_color.rgb * color.rgb;

    gl_FragColor = vec4(res, 1.0);
}
