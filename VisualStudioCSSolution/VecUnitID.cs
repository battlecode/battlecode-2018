//------------------------------------------------------------------------------
// <auto-generated />
//
// This file was automatically generated by SWIG (http://www.swig.org).
// Version 3.0.12
//
// Do not make changes to this file unless you know what you are doing--modify
// the SWIG interface file instead.
//------------------------------------------------------------------------------


public class VecUnitID : global::System.IDisposable {
  private global::System.Runtime.InteropServices.HandleRef swigCPtr;
  protected bool swigCMemOwn;

  internal VecUnitID(global::System.IntPtr cPtr, bool cMemoryOwn) {
    swigCMemOwn = cMemoryOwn;
    swigCPtr = new global::System.Runtime.InteropServices.HandleRef(this, cPtr);
  }

  internal static global::System.Runtime.InteropServices.HandleRef getCPtr(VecUnitID obj) {
    return (obj == null) ? new global::System.Runtime.InteropServices.HandleRef(null, global::System.IntPtr.Zero) : obj.swigCPtr;
  }

  ~VecUnitID() {
    Dispose();
  }

  public virtual void Dispose() {
    lock(this) {
      if (swigCPtr.Handle != global::System.IntPtr.Zero) {
        if (swigCMemOwn) {
          swigCMemOwn = false;
          bcPINVOKE.delete_VecUnitID(swigCPtr);
        }
        swigCPtr = new global::System.Runtime.InteropServices.HandleRef(null, global::System.IntPtr.Zero);
      }
      global::System.GC.SuppressFinalize(this);
    }
  }

  public VecUnitID() : this(bcPINVOKE.new_VecUnitID(), true) {
    if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
  }

  public string debug() {
    string ret = bcPINVOKE.VecUnitID_debug(swigCPtr);
    if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
    return ret;
  }

  public VecUnitID clone() {
    global::System.IntPtr cPtr = bcPINVOKE.VecUnitID_clone(swigCPtr);
    VecUnitID ret = (cPtr == global::System.IntPtr.Zero) ? null : new VecUnitID(cPtr, false);
    if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
    return ret;
  }

  public uint len() {
    uint ret = bcPINVOKE.VecUnitID_len(swigCPtr);
    if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
    return ret;
  }

  public ushort index(uint index) {
    ushort ret = bcPINVOKE.VecUnitID_index(swigCPtr, index);
    if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
    return ret;
  }

}
