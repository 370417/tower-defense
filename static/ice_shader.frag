precision mediump float;

varying vec2 vTextureCoord;
varying vec4 vColor;

uniform sampler2D uSampler;
uniform float customUniform;

void main(void)
{
    vec2 uvs = vTextureCoord.xy;

    vec4 fg = texture2D(uSampler, vTextureCoord);

    float value = fg.a > 0.35 && fg.r < customUniform * fg.a ? 1.0 : 0.0;

    // float blueness = smoothstep(customUniform * fg.a - 0.15, customUniform * fg.a, fg.r);
    float greenness = smoothstep(customUniform * fg.a - 0.15, customUniform * fg.a, fg.r);

    // gl_FragColor = vec4(value * (1.0 - 0.35 * blueness), value * (1.0 - 0.15 * blueness), value, value);
    gl_FragColor = vec4(value * (0.6 + 0.25 * greenness), value * (0.45 + 0.2 * greenness), value * (0.1 * greenness), value);
}
