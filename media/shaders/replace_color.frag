#version 100

precision lowp float;

varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;

uniform vec3 find_color;
uniform vec3 replace_color;

void main() {
    vec4 base_color = texture2D(Texture, uv);

    if (base_color.a != 1.0) {
        // Keep transparent pixels transparent.
        discard;
    }
    if (distance(find_color, base_color.rgb) < 0.01) {
        base_color.rgb = replace_color;
    }

    vec3 res = base_color.rgb * color.rgb;

    gl_FragColor = vec4(res, 1.0);
}
