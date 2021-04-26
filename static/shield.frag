precision mediump float;

varying vec2 vTextureCoord;
varying vec4 vColor;

uniform sampler2D uSampler;

// If the angle swept out by the shield is theta (in radians),
// then halfSize is theta / (4pi)
uniform float halfSize;

void main(void)
{
    vec2 uvs = vTextureCoord.xy;

    vec4 fg = texture2D(uSampler, vTextureCoord);

    float half_radius = 0.4;
    float half_thickness = 0.1;
    float epsilon = 0.01;

    float theta_value = smoothstep(halfSize - epsilon, halfSize, fg.g);
    float outer_r_value = smoothstep(half_radius + half_thickness, half_radius + half_thickness + epsilon, fg.r);
    float inner_r_value = 1.0 - smoothstep(half_radius - half_thickness - epsilon, half_radius - half_thickness, fg.r);

    float r_value = min(outer_r_value, inner_r_value);
    float value = min(r_value, thet_value);

    gl_FragColor = vec4(0.0, 0.0, 0.0, value);
}
