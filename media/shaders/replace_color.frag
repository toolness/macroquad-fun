#version 100

precision lowp float;

varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;

uniform int num_replacements;

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

uniform vec4 find_color_7;
uniform vec4 replace_color_7;

uniform vec4 find_color_8;
uniform vec4 replace_color_8;

uniform int lerp_type;
uniform vec4 lerp_color;
uniform float lerp_amount;

const int LERP_TYPE_NONE = 0;
const int LERP_TYPE_REPLACED_COLOR = 1;
const int LERP_TYPE_ALL_COLORS = 2;

bool replace_color(inout vec4 base_color, in vec4 find_color, in vec4 replace_color) {
    if (find_color.rgb == base_color.rgb) {
        if (replace_color.a != 1.0) {
            // Replace with a transparent pixel.
            discard;
        }
        base_color.rgb = replace_color.rgb;
        return true;
    }
    return false;
}

void main() {
    vec4 base_color = texture2D(Texture, uv);

    if (base_color.a != 1.0) {
        // Keep transparent pixels transparent.
        discard;
    }
    bool was_color_replaced = (num_replacements > 0 && replace_color(base_color, find_color_1, replace_color_1) ||
        (num_replacements > 1 && replace_color(base_color, find_color_2, replace_color_2) ||
            (num_replacements > 2 && replace_color(base_color, find_color_3, replace_color_3) ||
                (num_replacements > 3 && replace_color(base_color, find_color_4, replace_color_4) ||
                    (num_replacements > 4 && replace_color(base_color, find_color_5, replace_color_5) ||
                        (num_replacements > 5 && replace_color(base_color, find_color_6, replace_color_6) ||
                            (num_replacements > 6 && replace_color(base_color, find_color_7, replace_color_7) ||
                                (num_replacements > 7 && replace_color(base_color, find_color_8, replace_color_8))
                            )
                        )
                    )
                )
            )
        )
    );

    bool should_lerp = (lerp_type == LERP_TYPE_ALL_COLORS) || (lerp_type == LERP_TYPE_REPLACED_COLOR && was_color_replaced);

    if (should_lerp) {
        base_color = mix(base_color, lerp_color, lerp_amount);
    }

    vec3 res = base_color.rgb * color.rgb;

    gl_FragColor = vec4(res, 1.0);
}
