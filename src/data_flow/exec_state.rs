use crate::asm::kernel_code::{KernelCode, VGPRWorkItemId};
use crate::data_flow::types::{Binding, BuiltIn, Variable, Reg, Condition};

#[derive(Clone)]
pub struct ExecState {
    pub sgprs: Vec<Reg>,
    pub vgprs: Vec<Reg>,
    pub bindings: Vec<Binding>,
    pub variables: Vec<Variable>,
    pub vcc: Option<Condition>,
    pub scc: Option<Condition>
}

use std::fmt;

impl fmt::Debug for ExecState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Bindings:")?;
        self.bindings.iter().enumerate()
            .try_for_each(|(i, binding)| writeln!(f, "{:4} {:?}", i, binding))?;

        writeln!(f, "Variables:")?;
        self.variables.iter().enumerate()
            .try_for_each(|(i, variable)| writeln!(f, "{:4} {:?}", i, variable))?;

        writeln!(f, "SGPRS: {:?}", self.sgprs.iter().enumerate().collect::<Vec<(usize, &Reg)>>())?;
        writeln!(f, "VGPRS: {:?}", self.vgprs.iter().enumerate().collect::<Vec<(usize, &Reg)>>())?;
        writeln!(f, "SCC: {:?}, VCC: {:?}", self.scc, self.vcc)
    }
}

macro_rules! bind_init_state {
    (qword $val:expr, $bindings:expr, $regfile:expr) => {
        $bindings.push(Binding::InitState($val));
        $regfile.push(Reg($bindings.len() - 1, 0));
        $regfile.push(Reg($bindings.len() - 1, 1));
    };
    (dword $val:expr, $bindings:expr, $regfile:expr) => {
        $bindings.push(Binding::InitState($val));
        $regfile.push(Reg($bindings.len() - 1, 0));
    }
}

impl From<KernelCode> for ExecState {
    fn from(kcode: KernelCode) -> Self {
        let mut sgprs: Vec<Reg> = Vec::with_capacity(16);
        let mut bindings: Vec<Binding> = Vec::with_capacity(16);

        /* https://llvm.org/docs/AMDGPUUsage.html#amdgpu-amdhsa-sgpr-register-set-up-order-table */
        if kcode.code_props.enable_sgpr_private_segment_buffer {
            bindings.push(Binding::InitState(BuiltIn::PrivateSegmentBuffer));
            for i in 0..4 { sgprs.push(Reg(bindings.len() - 1, i)); }
        }
        if kcode.code_props.enable_sgpr_dispatch_ptr {
            bind_init_state!(qword BuiltIn::PtrDispatchPacket, bindings, sgprs);
        }
        if kcode.code_props.enable_sgpr_queue_ptr {
            bind_init_state!(qword BuiltIn::PtrQueue, bindings, sgprs);
        }
        if kcode.code_props.enable_sgpr_kernarg_segment_ptr {
            bind_init_state!(qword BuiltIn::PtrKernarg, bindings, sgprs);
        }
        if kcode.code_props.enable_sgpr_dispatch_id {
            bind_init_state!(qword BuiltIn::DispatchId, bindings, sgprs);
        }
        if kcode.code_props.enable_sgpr_flat_scratch_init {
            bind_init_state!(qword BuiltIn::FlatScratchInit, bindings, sgprs);
        }
        if kcode.code_props.enable_sgpr_grid_workgroup_count_x {
            bind_init_state!(dword BuiltIn::WorkgroupCountX, bindings, sgprs);
        }
        if kcode.code_props.enable_sgpr_grid_workgroup_count_y && sgprs.len() < 16 {
            bind_init_state!(dword BuiltIn::WorkgroupCountY, bindings, sgprs);
        }
        if kcode.code_props.enable_sgpr_grid_workgroup_count_z && sgprs.len() < 16 {
            bind_init_state!(dword BuiltIn::WorkgroupCountZ, bindings, sgprs);
        }
        if kcode.pgm_props.enable_sgpr_workgroup_id_x {
            bind_init_state!(dword BuiltIn::WorkgroupIdX, bindings, sgprs);
        }
        if kcode.pgm_props.enable_sgpr_workgroup_id_y {
            bind_init_state!(dword BuiltIn::WorkgroupIdY, bindings, sgprs);
        }
        if kcode.pgm_props.enable_sgpr_workgroup_id_z {
            bind_init_state!(dword BuiltIn::WorkgroupIdZ, bindings, sgprs);
        }
        if kcode.pgm_props.enable_sgpr_workgroup_info {
            bind_init_state!(dword BuiltIn::WorkgroupInfo, bindings, sgprs);
        }
        if kcode.pgm_props.enable_sgpr_private_segment_wavefront_offset {
            bind_init_state!(dword BuiltIn::PrivateSegmentWavefrontOffset, bindings, sgprs);
        }

        /* https://llvm.org/docs/AMDGPUUsage.html#amdgpu-amdhsa-vgpr-register-set-up-order-table */
        let vgprs: Vec<Reg> = match kcode.pgm_props.enable_vgpr_workitem_id {
            VGPRWorkItemId::X => {
                bindings.push(Binding::InitState(BuiltIn::WorkitemIdX));
                vec![Reg(bindings.len() - 1, 0)]
            },
            VGPRWorkItemId::XY => {
                bindings.push(Binding::InitState(BuiltIn::WorkitemIdX));
                bindings.push(Binding::InitState(BuiltIn::WorkitemIdY));
                vec![Reg(bindings.len() - 2, 0), Reg(bindings.len() - 1, 0)]
            },
            VGPRWorkItemId::XYZ => {
                bindings.push(Binding::InitState(BuiltIn::WorkitemIdX));
                bindings.push(Binding::InitState(BuiltIn::WorkitemIdY));
                bindings.push(Binding::InitState(BuiltIn::WorkitemIdZ));
                vec![Reg(bindings.len() - 3, 0), Reg(bindings.len() - 2, 0), Reg(bindings.len() - 1, 0)]
            }
        };
        
        ExecState { sgprs, vgprs, bindings, scc: None, vcc: None, variables: Vec::new() }
    }
}
