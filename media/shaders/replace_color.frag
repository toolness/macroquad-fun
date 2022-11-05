#version 100

precision lowp float;

varying vec4 color;
varying vec2 uv;
    
uniform sampler2D Texture;

void main() {
    vec3 red = vec3(255.0, 24.0, 49.0) / 255.0;
    vec3 blue = vec3(0.0, 0.0, 255.0) / 255.0;
    vec4 base_color = texture2D(Texture, uv);

    if (base_color.a != 1.0) {
        // Keep transparent pixels transparent.
        discard;
    }
    if (length(red - base_color.rgb) < 0.01) {
        base_color.rgb = blue;
    }

    vec3 res = base_color.rgb * color.rgb;

    gl_FragColor = vec4(res, 1.0);
}
