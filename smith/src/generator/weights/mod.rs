pub mod expr;
pub mod stmt;
pub mod type_weights;

trait EnumWeights: Sized + 'static {
    fn all() -> &'static [Self];

    fn weights() -> Vec<u32> {
        Self::all().iter().map(|x| x.weight()).collect::<Vec<u32>>()
    }

    fn weight(&self) -> u32;
}
