struct VsOutput {
	float4 pos: SV_Position;
	float2 uv: TEXCOORD;
};

Texture2D<float> t_Src;
SamplerState t_Src_;

cbuffer Locals {
	float2 u_InvSize;
	bool u_Init;
};

VsOutput VS(float2 pos: a_Pos, float2 uv: a_TexCoord) {
	VsOutput p = {
		float4(pos, 0.0, 1.0),
		uv,
	};
	return p;
}

float4 PS(VsOutput v): SV_Target {
	if (u_Init) {
		float2 pixel = v.uv/u_InvSize * float2(19, 127);
		if ((pixel.x*(pixel.y%100) + pixel.x*53 + pixel.y)%13 > 6) {
			return float4(1.0.xxx, 1.0);
		} else {
			return float4(0.0.xxx, 1.0);
		}
	}

	float4 prev = t_Src.Sample(t_Src_, v.uv);
	float4 neighbours = 
		t_Src.Sample(t_Src_, v.uv + float2(-u_InvSize.x, -u_InvSize.y)) + 
		t_Src.Sample(t_Src_, v.uv + float2(0.0, -u_InvSize.y)) +
		t_Src.Sample(t_Src_, v.uv + float2(u_InvSize.x, -u_InvSize.y)) +
		t_Src.Sample(t_Src_, v.uv + float2(-u_InvSize.x, 0.0)) +
		t_Src.Sample(t_Src_, v.uv + float2(u_InvSize.x, 0.0)) +
		t_Src.Sample(t_Src_, v.uv + float2(-u_InvSize.x, u_InvSize.y)) +
		t_Src.Sample(t_Src_, v.uv + float2(0.0, u_InvSize.y)) +
		t_Src.Sample(t_Src_, v.uv + float2(u_InvSize.x, u_InvSize.y));

	if (neighbours.x == 2) {
		return prev;
	} else if (neighbours.x == 3) {
		return float4(1.0.xxx, 1.0);
	} else {
		return float4(0.0.xxx, 1.0);
	}
}

float4 PS_Display(VsOutput v): SV_Target {
	float4 cell = t_Src.Sample(t_Src_, v.uv);
	return float4(cell.xxx, 1.0);
}
