import { readFileSync } from 'fs';

const testCases = [
    `shader S by U {
    @vertex
    micro vs_main(position: vec2) -> vec4f {
        return vec4(1.0, 0.0, 0.0, 1.0)
    }
}`,
    `shader S by U {
    let albedo: texture = "white"
    @vertex
    micro vs_main() -> vec4f {
        return vec4(1.0, 1.0, 1.0, 1.0)
    }
}`,
    `shader S by U {
    structure Uniforms {
        mvp: mat44
    }
    @vertex
    micro vs_main() -> vec4f {
        return vec4(1.0, 1.0, 1.0, 1.0)
    }
}`,
    `shader S by U {
    @vertex
    micro vs_main() -> vec4f {
        vec4(1.0, 1.0, 1.0, 1.0)
    }
}`,
    `shader S by U {
    @vertex
    micro vs_main() -> f32 {
        1.0
    }
}`,
    `shader S by U {
    @vertex
    micro vs_main() -> f32 {
        return 1.0
    }
}`,
];

for (let i = 0; i < testCases.length; i++) {
    console.log(`\n=== Test case ${i + 1} ===`);
    console.log(testCases[i]);
    console.log(`Length: ${testCases[i].length}`);
}
